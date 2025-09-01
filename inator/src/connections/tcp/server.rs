use std::net::{IpAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use crate::connections::{BytesOptions, Connection, OrderOptions};

pub struct ServerTcpSettings {
    pub address: IpAddr,
    pub port: u16,
    pub bytes: BytesOptions,
    pub order: OrderOptions,
    pub max_connections: usize,
    pub recuse_when_full: bool
}

pub struct ServerTcpConnection{
    pub settings: ServerTcpSettings,
    pub name: &'static str,
    pub listener: Option<Arc<TcpListener>>,
    pub started: bool,
    pub runtime: Option<Runtime>,
    pub dropped: Arc<AtomicBool>,
    pub cancel_token: Arc<CancellationToken>,
    pub connection_down_sender: Arc<UnboundedSender<()>>,
    pub connection_down_receiver: UnboundedReceiver<()>,
    pub connection_up_sender: Arc<UnboundedSender<Arc<TcpListener>>>,
    pub connection_up_receiver: UnboundedReceiver<Arc<TcpListener>>,
}

impl ServerTcpConnection {
    pub fn new(settings: ServerTcpSettings, name: &'static str) -> ServerTcpConnection {
        let (connection_down_sender,connection_down_receiver) = unbounded_channel::<()>();
        let (connection_up_sender, connection_up_receiver) = unbounded_channel::<Arc<TcpListener>>();

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
            connection_up_receiver
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