use std::net::IpAddr;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use crate::connections::{BytesOptions, Connection, OrderOptions};

pub struct ClientTcpSettings {
    pub(crate) address: IpAddr,
    pub(crate) port: u16,
    pub(crate) bytes: BytesOptions,
    pub(crate) order: OrderOptions,
    pub(crate) max_connections: usize,
    pub(crate) recuse_when_full: bool
}
pub struct ClientTcpConnection {
    pub(crate) settings: ClientTcpSettings,
    pub(crate) name: &'static str,
    pub(crate) started: bool,
    pub(crate) read_half: Option<OwnedReadHalf>,
    pub(crate) write_half: Option<OwnedWriteHalf>
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
