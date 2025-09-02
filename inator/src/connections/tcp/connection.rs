use std::net::SocketAddr;
use std::sync::Arc;
use bincode::config::standard;
use bincode::serde::encode_to_vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use crate::connections::{BytesOptions, OrderOptions};
use crate::connections::tcp::reader_writer::{read_from_settings, value_to_bytes, write_from_settings};
use crate::NetworkSide;
use crate::systems::messaging::{deserialize_message, MessageTrait};

pub struct TcpConnection {
    pub read_half: Option<Arc<Mutex<OwnedReadHalf>>>,
    pub write_half: Option<Arc<Mutex<OwnedWriteHalf>>>,
    pub connection_name: &'static str,
    pub socket_addr: SocketAddr,
    pub network_side: NetworkSide,
    pub uuid: Option<Uuid>,
    pub cancellation_token: Arc<CancellationToken>,
    pub connection_down_sender: Arc<UnboundedSender<()>>,
    pub connection_down_receiver: UnboundedReceiver<()>,
    pub message_received_sender: Arc<UnboundedSender<Box<dyn MessageTrait>>>,
    pub message_received_receiver: UnboundedReceiver<Box<dyn MessageTrait>>,
    pub bytes: BytesOptions,
    pub order: OrderOptions
}

impl TcpConnection {
    pub fn new(tcp_stream: TcpStream, connection_name: &'static str, network_side: NetworkSide, cancellation_token: Arc<CancellationToken>, bytes: BytesOptions, order: OrderOptions) -> Self {
        let socket_addr = tcp_stream.peer_addr().unwrap();
        let (connection_down_sender, connection_down_receiver) = unbounded_channel::<()>();
        let (message_received_sender, message_received_receiver) = unbounded_channel::<Box<dyn MessageTrait>>();
        let (read_half, write_half) = match tcp_stream.into_split() {
            (read_half, write_half) => (Some(Arc::new(Mutex::new(read_half))), Some(Arc::new(Mutex::new(write_half)))),
        };

        TcpConnection {
            read_half,
            write_half,
            connection_name,
            socket_addr,
            network_side,
            uuid: if network_side == NetworkSide::Server {Some(Uuid::new_v4())} else {None},
            cancellation_token: Arc::clone(&cancellation_token),
            connection_down_sender: Arc::new(connection_down_sender),
            connection_down_receiver,
            message_received_sender: Arc::new(message_received_sender),
            message_received_receiver,
            bytes,
            order
        }
    }

    pub fn start_listen_server(&mut self, runtime: &Runtime) {
        assert!(self.network_side == NetworkSide::Server,"You can just listen the server from client");

        let read_half = match &self.read_half {
            Some(read_half) => Arc::clone(read_half),
            None => return,
        };

        let connection_down_sender = Arc::clone(&self.connection_down_sender);
        let message_received_sender = Arc::clone(&self.message_received_sender);
        let bytes_options = self.bytes;
        let order_options = self.order;
        let cancellation_token = Arc::clone(&self.cancellation_token);

        runtime.spawn(async move {
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    },
                    mut guard = read_half.lock() => {
                        match read_from_settings(&mut guard, &bytes_options, &order_options).await {
                            Ok(read_value) => {
                                let mut buf = value_to_bytes(&read_value, &order_options);

                                match guard.read_exact(&mut buf).await {
                                    Ok(_) => {
                                        if let Some(message) = deserialize_message(&buf) {
                                            message_received_sender.send(message).unwrap();
                                        } else {
                                            println!("Mensage not registered or failed to deserialize message");
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

    pub fn send_client_message(&mut self, message: Box<dyn MessageTrait>, runtime: &Runtime) {
        assert!(self.network_side == NetworkSide::Client,"You can just send a client message on server");

        let write_half = match &self.write_half {
            Some(write_half) => Arc::clone(write_half),
            None => return,
        };

        let bytes_options = self.bytes;
        let order_options = self.order;
        let config = standard();
        let encoded = encode_to_vec(&message, config).unwrap();

        runtime.spawn(async move {
            let mut guard = write_half.lock().await;

            match write_from_settings(&mut guard,&encoded,&bytes_options,&order_options).await {
                Ok(_) => {
                    match guard.write_all(&encoded).await {
                        Ok(_) => {
                            println!("Message sent for client")
                        },
                        Err(e) => {
                            println!("Error on send: {:?}", e);
                        }
                    }
                }
                Err(_) => {

                }
            }
        });
    }
}