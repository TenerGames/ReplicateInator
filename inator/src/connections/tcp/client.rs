use std::net::IpAddr;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
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
    pub read_half: Option<OwnedReadHalf>,
    pub write_half: Option<OwnedWriteHalf>
}

impl ClientTcpConnection {
    pub fn new(settings: ClientTcpSettings, name: &'static str) -> ClientTcpConnection {
        ClientTcpConnection {
            settings,
            name,
            started: false,
            read_half: None,
            write_half: None
        }
    }
}

impl Connection for ClientTcpConnection {
    fn start_connection(&mut self) {
        todo!()
    }

    fn can_start(&self) -> bool {
        !&self.started
    }

    fn disconnect(&mut self) {
        todo!()
    }
}