use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use crate::connections::{BytesOptions, Connection, OrderOptions};
use crate::connections::tcp::connection::TcpConnection;

pub struct ServerTcpSettings {
    pub(crate) address: IpAddr,
    pub(crate) port: u16,
    pub(crate) bytes: BytesOptions,
    pub(crate) order: OrderOptions,
    pub(crate) max_connections: usize,
    pub(crate) recuse_when_full: bool
}

pub struct ServerTcpConnection{
    pub(crate) settings: ServerTcpSettings,
    pub(crate) name: &'static str,
    pub(crate) listener: Option<Arc<TcpListener>>,
    pub(crate) started: bool,
    pub(crate) runtime: Option<Runtime>,
    pub(crate) dropped: Arc<AtomicBool>,
    pub(crate) cancel_token: Arc<CancellationToken>,
    pub(crate) connection_down_sender: Arc<UnboundedSender<()>>,
    pub(crate) connection_down_receiver: UnboundedReceiver<()>,
    pub(crate) connection_up_sender: Arc<UnboundedSender<Arc<TcpListener>>>,
    pub(crate) connection_up_receiver: UnboundedReceiver<Arc<TcpListener>>,
    pub(crate) client_connected_sender: Arc<UnboundedSender<(TcpStream,SocketAddr)>>,
    pub(crate) client_connected_receiver: UnboundedReceiver<(TcpStream,SocketAddr)>,
    pub(crate) connections: HashMap<Uuid,TcpConnection>
}

impl Default for ServerTcpSettings {
    fn default() -> Self {
        ServerTcpSettings {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
            bytes: BytesOptions::U32,
            order: OrderOptions::LittleEndian,
            max_connections: 0,
            recuse_when_full: false
        }
    }
}

impl ServerTcpConnection {
    pub fn new(settings: ServerTcpSettings, name: &'static str) -> ServerTcpConnection {
        let (connection_down_sender,connection_down_receiver) = unbounded_channel::<()>();
        let (connection_up_sender, connection_up_receiver) = unbounded_channel::<Arc<TcpListener>>();
        let (client_connected_sender, client_connected_receiver) = unbounded_channel::<(TcpStream,SocketAddr)>();

        ServerTcpConnection {
            settings,
            name,
            listener: None,
            started: false,
            runtime: Some(Runtime::new().unwrap()),
            dropped: Arc::new(AtomicBool::new(false)),
            cancel_token: Arc::new(CancellationToken::new()),
            connection_down_sender: Arc::new(connection_down_sender),
            connection_down_receiver,
            connection_up_sender: Arc::new(connection_up_sender),
            connection_up_receiver,
            client_connected_sender: Arc::new(client_connected_sender),
            client_connected_receiver,
            connections: HashMap::new()
        }
    }
}

impl Connection for ServerTcpConnection {
    fn start_connection(&mut self) {
        if !self.can_start() {return;}

        let settings = &self.settings;
        let max_connections = settings.max_connections;
        let address = (settings.address, settings.port);
        let dropped = Arc::clone(&self.dropped);
        let recuse_when_full = settings.recuse_when_full;
        let connection_up_sender = Arc::clone(&self.connection_up_sender);
        let connection_down_sender = Arc::clone(&self.connection_down_sender);
        let client_connected_sender = Arc::clone(&self.client_connected_sender);
        let cancel_token = Arc::clone(&self.cancel_token);

        self.started = true;

        self.runtime.as_ref().unwrap().spawn(async move {
            dropped.store(false, Ordering::SeqCst);

            let tcp_listener = loop {
                match TcpListener::bind(address).await {
                    Ok(listener) => break Arc::new(listener),
                    Err(e) => {
                        println!("Error on bind: {}, trying again...", e);

                        if dropped.load(Ordering::SeqCst) {
                            return;
                        }

                        continue;
                    }
                };
            };

            if dropped.load(Ordering::SeqCst) {
                drop(tcp_listener);
                return;
            }

            println!("Server tcp binded successfully!");

            connection_up_sender.send(Arc::clone(&tcp_listener)).unwrap();

            let semaphore: Option<Semaphore> = if max_connections > 0 { Some(Semaphore::new(max_connections)) } else { None };

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        break;
                    },
                    accept_result = tcp_listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                match &semaphore {
                                    Some(semaphore) => {
                                        if recuse_when_full {
                                            match semaphore.try_acquire() {
                                                Ok(permit) => {
                                                    println!("Accepted connection from {}", addr);

                                                    client_connected_sender.send((stream,addr)).unwrap();

                                                    drop(permit);
                                                },
                                                Err(_) => {
                                                    println!("Connection is full, rejecting connection to {}", addr);

                                                    drop(stream);
                                                }
                                            }
                                        }else {
                                            match semaphore.acquire().await {
                                                Ok(permit) => {
                                                    println!("Accepted connection from {}", addr);

                                                    client_connected_sender.send((stream,addr)).unwrap();

                                                    drop(permit);
                                                },
                                                Err(_) => {
                                                    drop(stream);
                                                }
                                            }
                                        }
                                    },
                                    None => {
                                        println!("Accepted connection from {}", addr);

                                        client_connected_sender.send((stream,addr)).unwrap();
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Error on accept: {:?}", e);

                                match e.kind() {
                                    std::io::ErrorKind::ConnectionAborted => {
                                         println!("Listener aborted (network down or aborted by OS)");

                                         connection_down_sender.send(()).unwrap();

                                         break;
                                    },
                                    std::io::ErrorKind::Other => {
                                         println!("Listener was probably closed manually");

                                         connection_down_sender.send(()).unwrap();

                                         break;
                                    },
                                    _ => {
                                        println!("Unexpected error: {:?}", e.kind());
                                    }
                                }
                            }
                        }
                    }
                }

                if dropped.load(Ordering::SeqCst) {
                    drop(tcp_listener);
                    break;
                }
            }
        });
    }

    fn can_start(&self) -> bool {
        !&self.started
    }

    fn cancel_connection(&mut self) {
        if let Some(listener) = self.listener.take() {
            drop(listener);
        }

        self.cancel_token.cancel();
        self.dropped.store(true,Ordering::SeqCst);
        self.started = false;
    }

    fn disconnect(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }

        if let Some(listener) = self.listener.take() {
            drop(listener);
        }

        self.cancel_token.cancel();
        self.dropped.store(true, Ordering::SeqCst);
    }
}