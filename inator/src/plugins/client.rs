use std::sync::Arc;
use bevy::app::App;
use bevy::prelude::{First, IntoScheduleConfigs, Last, Plugin, ResMut};
use crate::connections::{ClientConnectionType, ClientConnections, Connection, Connections};
use crate::connections::tcp::connection::TcpConnection;
use crate::NetworkSide;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientConnections::new());
        app.add_systems(First,start_connections);
        app.add_systems(Last,(check_connection_up,restart_connection).chain());
    }
}

pub fn start_connections(
    mut client_connections: ResMut<ClientConnections>,
){
    for (_,connection) in client_connections.0.iter_mut() {
        match connection {
            ClientConnectionType::Tcp(connection) => {
                connection.start_connection()
            }
        }
    }
}

pub fn check_connection_up(
    mut client_connections: ResMut<ClientConnections>,
){
    for (_,connection) in client_connections.0.iter_mut() {
        match connection {
            ClientConnectionType::Tcp(connection) => {
                match connection.connection_up_receiver.try_recv() {
                    Ok(tcp_stream) => {
                        if let Some(local_tcp_connection) = connection.local_tcp_connection.take() {
                            drop(local_tcp_connection);
                        }

                        let mut tcp_connection = TcpConnection::new(tcp_stream, connection.name, NetworkSide::Client);

                        tcp_connection.start_listen_server(connection.runtime.as_ref().unwrap(),Arc::clone(&connection.cancel_token), &connection.settings);

                        connection.local_tcp_connection = Some(tcp_connection);
                    }
                    Err(_) => {
                        continue
                    }
                }
            }
        }
    }
}

pub fn restart_connection(
    mut client_connections: ResMut<ClientConnections>,
){
    for (_,connection) in client_connections.0.iter_mut() {
        match connection {
            ClientConnectionType::Tcp(connection) => {
                match connection.local_tcp_connection.as_mut() {
                    Some(local_tcp_connection) => {
                        match local_tcp_connection.connection_down_receiver.try_recv() {
                            Ok(_) => {
                                connection.cancel_connection()
                            }
                            Err(_) => {}
                        }
                    }
                    None => {
                        continue
                    }
                }
            }
        }
    }
}