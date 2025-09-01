use std::sync::Arc;
use bevy::app::App;
use bevy::prelude::{EventWriter, First, IntoScheduleConfigs, Last, Plugin, ResMut, Update};
use crate::connections::{Connection, Connections, ServerConnectionType, ServerConnections};
use crate::connections::tcp::connection::TcpConnection;
use crate::NetworkSide;
use crate::plugins::{ClientConnected, ConnectedMessage};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerConnections::new());
        app.add_event::<ClientConnected>();
        app.add_systems(First,start_connections);
        app.add_systems(Update,check_clients_connected);
        app.add_systems(Last,(check_connection_up,restart_connection).chain());
    }
}

pub fn start_connections(
    mut server_connections: ResMut<ServerConnections>,
){
    for (_,connection) in server_connections.0.iter_mut() {
        match connection {
            ServerConnectionType::Tcp(connection) => {
                connection.start_connection()
            }
        }
    }
}

pub fn check_clients_connected(
    mut server_connections: ResMut<ServerConnections>,
    mut client_connected_event: EventWriter<ClientConnected>,
){
    for (_,connection) in server_connections.0.iter_mut() {
        match connection {
            ServerConnectionType::Tcp(connection) => {
                match connection.client_connected_receiver.try_recv() {
                    Ok((tcp_stream,addr)) => {
                        client_connected_event.write(ClientConnected(addr,connection.name));

                        let settings = &connection.settings;
                        let mut tcp_connection = TcpConnection::new(tcp_stream, connection.name, NetworkSide::Client, Arc::clone(&connection.cancel_token),settings.bytes,settings.order);
                        let current_uuid = tcp_connection.uuid.unwrap();
                        
                        tcp_connection.send_client_message(Box::new(ConnectedMessage{
                            uuid: current_uuid
                        }),connection.runtime.as_ref().unwrap());
                        
                        connection.connections.insert(current_uuid,tcp_connection);
                    }
                    Err(_) => {
                        continue
                    }
                }
            }
        }
    }
}

pub fn check_connection_up(
    mut server_connections: ResMut<ServerConnections>,
){
    for (_,connection) in server_connections.0.iter_mut() {
        match connection {
            ServerConnectionType::Tcp(connection) => {
                match connection.connection_up_receiver.try_recv() {
                    Ok(tcp_listener) => {
                        if let Some(listener) = connection.listener.take() {
                            drop(listener);
                        }

                        connection.listener = Some(tcp_listener);
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
    mut server_connections: ResMut<ServerConnections>,
){
    for (_,connection) in server_connections.0.iter_mut() {
        match connection {
            ServerConnectionType::Tcp(connection) => {
                match connection.connection_down_receiver.try_recv() {
                    Ok(_) => {
                        connection.cancel_connection()
                    }
                    Err(_) => {
                        continue
                    }
                }
            }
        }
    }
}