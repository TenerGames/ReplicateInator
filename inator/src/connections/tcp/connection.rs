use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use crate::NetworkSide;

pub struct TcpConnection {
    pub read_half: Option<Arc<Mutex<OwnedReadHalf>>>,
    pub write_half: Option<Arc<OwnedWriteHalf>>,
    pub connection_name: &'static str,
    pub socket_addr: SocketAddr,
    pub network_side: NetworkSide,
    pub connection_down_sender: Arc<UnboundedSender<()>>,
    pub connection_down_receiver: UnboundedReceiver<()>,
}

impl TcpConnection {
    pub fn new(tcp_stream: TcpStream, connection_name: &'static str, network_side: NetworkSide) -> Self {
        let socket_addr = tcp_stream.peer_addr().unwrap();
        let (connection_down_sender, connection_down_receiver) = unbounded_channel::<()>();
        let (read_half, write_half) = match tcp_stream.into_split() {
            (read_half, write_half) => (Some(Arc::new(Mutex::new(read_half))), Some(Arc::new(write_half))),
        };

        TcpConnection {
            read_half,
            write_half,
            connection_name,
            socket_addr,
            network_side,
            connection_down_sender: Arc::new(connection_down_sender),
            connection_down_receiver
        }
    }

    pub fn start_listen_server(&mut self, runtime: &Runtime, cancellation_token: Arc<CancellationToken>) {
        assert!(self.network_side == NetworkSide::Server,"You can just listen the server from client");

        let read_half = match &self.read_half {
            Some(read_half) => Arc::clone(read_half),
            None => return,
        };

        let connection_down_sender = Arc::clone(&self.connection_down_sender);

        runtime.spawn(async move {
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    },
                    mut guard = read_half.lock() => {
                        match guard.read_u32().await {
                            Ok(length) => {
                                let mut buf = vec![0u8; length as usize];

                                match guard.read_exact(&mut buf).await {
                                    Ok(_) => {

                                    },
                                    Err(e) => {
                                        eprintln!("Error on read: {:?}", e);

                                        match e.kind() {
                                            std::io::ErrorKind::ConnectionAborted => {

                                                println!("Connection aborted (network down or aborted by OS)");

                                                connection_down_sender.send(()).unwrap();

                                                break;
                                            },
                                            std::io::ErrorKind::Other => {

                                                println!("Connection was probably closed manually");

                                                connection_down_sender.send(()).unwrap();

                                                break;
                                            },

                                            _ => {

                                                println!("Unexpected error: {:?}", e.kind());
                                            }

                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Error on read: {:?}", e);

                                match e.kind() {
                                    std::io::ErrorKind::ConnectionAborted => {
                                         println!("Connection aborted (network down or aborted by OS)");

                                         connection_down_sender.send(()).unwrap();

                                         break;
                                    },
                                    std::io::ErrorKind::Other => {
                                         println!("Connection was probably closed manually");

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
            }
        });
    }
}