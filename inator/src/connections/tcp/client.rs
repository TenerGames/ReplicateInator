use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use crate::connections::{BytesOptions, Connection, OrderOptions};

pub struct ClientTcpSettings {
    pub address: IpAddr,
    pub port: u16,
    pub bytes: BytesOptions,
    pub order: OrderOptions,
    pub max_connections: usize,
    pub recuse_when_full: bool
}
pub struct ClientTcpConnection {
    pub settings: ClientTcpSettings,
    pub name: &'static str,
    pub started: bool,
    pub runtime: Option<Runtime>,
    pub dropped: Arc<AtomicBool>,
    pub cancel_token: Arc<CancellationToken>,
    pub read_half: Option<Arc<Mutex<OwnedReadHalf>>>,
    pub write_half: Option<Arc<OwnedWriteHalf>>,
    pub connection_down_sender: Arc<UnboundedSender<()>>,
    pub connection_down_receiver: UnboundedReceiver<()>,
    pub connection_up_sender: Arc<UnboundedSender<(Arc<Mutex<OwnedReadHalf>>,Arc<OwnedWriteHalf>)>>,
    pub connection_up_receiver: UnboundedReceiver<(Arc<Mutex<OwnedReadHalf>>,Arc<OwnedWriteHalf>)>,
}

impl ClientTcpConnection {
    pub fn new(settings: ClientTcpSettings, name: &'static str) -> ClientTcpConnection {
        let (connection_down_sender,connection_down_receiver) = unbounded_channel::<()>();
        let (connection_up_sender, connection_up_receiver) = unbounded_channel::<(Arc<Mutex<OwnedReadHalf>>,Arc<OwnedWriteHalf>)>();

        ClientTcpConnection {
            settings,
            name,
            started: false,
            runtime: Some(Runtime::new().unwrap()),
            dropped: Arc::new(AtomicBool::new(false)),
            cancel_token: Arc::new(CancellationToken::new()),
            read_half: None,
            write_half: None,
            connection_down_sender: Arc::new(connection_down_sender),
            connection_down_receiver,
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

            let (read_half, write_half) = match tcp_stream.into_split() {
                (read_half, write_half) => (Arc::new(Mutex::new(read_half)), Arc::new(write_half)),
            };

            connection_up_sender.send((Arc::clone(&read_half),Arc::clone(&write_half))).unwrap();
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

        if let Some(read_half) = self.read_half.take() {
            drop(read_half);
        }

        if let Some(write_half) = self.write_half.take() {
            drop(write_half);
        }

        self.cancel_token.cancel();
        self.dropped.store(true, Ordering::SeqCst);
    }
}