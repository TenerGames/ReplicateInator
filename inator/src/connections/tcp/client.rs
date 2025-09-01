use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::{TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use crate::connections::{BytesOptions, Connection, OrderOptions};
use crate::connections::tcp::connection::TcpConnection;

pub struct ClientTcpSettings {
    pub address: IpAddr,
    pub port: u16,
    pub bytes: BytesOptions,
    pub order: OrderOptions
}
pub struct ClientTcpConnection {
    pub settings: ClientTcpSettings,
    pub name: &'static str,
    pub started: bool,
    pub runtime: Option<Runtime>,
    pub dropped: Arc<AtomicBool>,
    pub local_tcp_connection: Option<TcpConnection>,
    pub cancel_token: Arc<CancellationToken>,
    pub connection_up_sender: Arc<UnboundedSender<TcpStream>>,
    pub connection_up_receiver:  UnboundedReceiver<TcpStream>
}

impl Default for ClientTcpSettings {
    fn default() -> Self {
        ClientTcpSettings {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
            bytes: BytesOptions::U32,
            order: OrderOptions::LittleEndian
        }
    }
}

impl ClientTcpConnection {
    pub fn new(settings: ClientTcpSettings, name: &'static str) -> ClientTcpConnection {
        let (connection_up_sender, connection_up_receiver) = unbounded_channel::<TcpStream>();

        ClientTcpConnection {
            settings,
            name,
            started: false,
            runtime: Some(Runtime::new().unwrap()),
            dropped: Arc::new(AtomicBool::new(false)),
            local_tcp_connection: None,
            cancel_token: Arc::new(CancellationToken::new()),
            connection_up_sender: Arc::new(connection_up_sender),
            connection_up_receiver
        }
    }
}

impl Connection for ClientTcpConnection {
    fn start_connection(&mut self) {
        if !self.can_start() {return;}

        let settings = &self.settings;
        let address = (settings.address, settings.port);
        let dropped = Arc::clone(&self.dropped);
        let connection_up_sender = Arc::clone(&self.connection_up_sender);

        self.started = true;

        self.runtime.as_ref().unwrap().spawn(async move {
            dropped.store(false, Ordering::SeqCst);

            let tcp_stream = loop {
                match TcpStream::connect(address).await {
                    Ok(stream) => break stream,
                    Err(e) => {
                        println!("Failed to connect to server: {}", e);

                        if dropped.load(Ordering::SeqCst) {
                            return;
                        }

                        continue;
                    }
                }
            };

            connection_up_sender.send(tcp_stream).unwrap();
        });
    }

    fn can_start(&self) -> bool {
        !&self.started
    }

    fn cancel_connection(&mut self) {
        if let Some(local_tcp_connection) = self.local_tcp_connection.take() {
            drop(local_tcp_connection);
        }

        self.cancel_token.cancel();
        self.dropped.store(true,Ordering::SeqCst);
        self.started = false;
    }

    fn disconnect(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }

        if let Some(local_tcp_connection) = self.local_tcp_connection.take() {
            drop(local_tcp_connection);
        }

        self.cancel_token.cancel();
        self.dropped.store(true, Ordering::SeqCst);
    }
}