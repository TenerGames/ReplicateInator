use std::collections::HashMap;
use bevy::log::warn;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use crate::connections::tcp::client::{ClientTcpConnection, ClientTcpSettings};
use crate::connections::tcp::server::{ServerTcpConnection, ServerTcpSettings};
pub mod tcp;

type ConnectMap<T> = HashMap<String,T>;

#[derive(Resource)]
pub struct ClientConnections(pub ConnectMap<ClientConnectionType>);

#[derive(Resource)]
pub struct ServerConnections(pub ConnectMap<ServerConnectionType>);

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum OrderOptions{
    LittleEndian,
    BigEndian
}

#[derive(Debug, Clone, Copy)]
pub enum BytesOptions {
    U8,
    U16,
    U32,
    U64,
    U128,

    I8,
    I16,
    I32,
    I64,
    I128,

    F32,
    F64,
}


#[derive(Debug)]
pub enum ReadValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    F32(f32),
    F64(f64),
}

#[derive(Serialize, Deserialize)]
pub enum ConnectionsType{
    Tcp
}

pub enum ServerConnectionType{
    Tcp(ServerTcpConnection)
}

pub enum ClientConnectionType{
    Tcp(ClientTcpConnection)
}

pub trait Connection {
    fn start_connection(&mut self);
    fn can_start(&self) -> bool;
    fn cancel_connection(&mut self);
    fn disconnect(&mut self);
}

pub trait Connections {
    fn new() -> Self;
    fn remove_connection(&mut self, name: &str);
    fn new_server_tcp_connection(&mut self, settings: ServerTcpSettings, name: &'static str);
    fn new_client_tcp_connection(&mut self, settings: ClientTcpSettings, name: &'static str);
}

impl Default for BytesOptions{
    fn default() -> BytesOptions{
        BytesOptions::U32
    }
}

impl Default for OrderOptions{
    fn default() -> OrderOptions{
        OrderOptions::LittleEndian
    }
}

impl Connections for ClientConnections {
    fn new() -> ClientConnections {
        ClientConnections(HashMap::new())
    }

    fn remove_connection(&mut self, name: &str) {
        if let Some(connection) = self.0.remove(name) {
            match connection {
                ClientConnectionType::Tcp(mut connection) => {
                    connection.disconnect();
                }
            }
        }
    }

    fn new_server_tcp_connection(&mut self, _settings: ServerTcpSettings, _name: &'static str) {
        warn!("You cant create server connection on client");
    }

    fn new_client_tcp_connection(&mut self, settings: ClientTcpSettings, name: &'static str) {
        if self.0.contains_key(name) {
            warn!("You already have a connection with this name");
            return;
        }

        let parsed_name = match name.parse::<String>() {
            Ok(name) => name,
            Err(_) => {
                warn!("Invalid string name");
                return;
            }
        };

        self.0.insert(parsed_name, ClientConnectionType::Tcp(ClientTcpConnection::new(settings, name)));
    }
}

impl Connections for ServerConnections {
    fn new() -> ServerConnections {
        ServerConnections(HashMap::new())
    }

    fn remove_connection(&mut self, name: &str) {
        if let Some(connection) = self.0.remove(name) {
            match connection {
                ServerConnectionType::Tcp(mut connection) => {
                    connection.disconnect();
                }
            }
        }
    }

    fn new_server_tcp_connection(&mut self, settings: ServerTcpSettings, name: &'static str) {
        if self.0.contains_key(name) {
            warn!("You already have a connection with this name");
            return;
        }

        let parsed_name = match name.parse::<String>() {
            Ok(name) => name,
            Err(_) => {
                warn!("Invalid string name");
                return;
            }
        };

        self.0.insert(parsed_name, ServerConnectionType::Tcp(ServerTcpConnection::new(settings, name)));
    }

    fn new_client_tcp_connection(&mut self, _settings: ClientTcpSettings, _name: &'static str) {
        warn!("You cant create client connection on server");
    }
}
