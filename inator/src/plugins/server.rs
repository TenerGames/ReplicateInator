use std::any::Any;
use std::sync::Arc;
use bevy::app::App;
use bevy::prelude::{Commands, First, IntoScheduleConfigs, Last, MessageWriter, Plugin, ResMut, Update, World};
use uuid::Uuid;
use crate::connections::{Connection, Connections, ConnectionsType, ServerConnectionType, ServerConnections};
use crate::connections::tcp::connection::TcpConnection;
use crate::NetworkSide;
use crate::plugins::{ClientConnected, ClientDiconnected, ConnectedMessage};
use crate::plugins::replication::{NewClientsToReplicate};
use crate::systems::messaging::{register_message_type, DISPATCHERS};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        register_message_type::<ConnectedMessage>(app, &NetworkSide::Client);

        app.insert_resource(ServerConnections::new());
        app.add_message::<ClientConnected>();
        app.add_message::<ClientDiconnected>();
        app.add_systems(First,(start_connections,check_client_connections_down).chain());
        app.add_systems(Update,(check_clients_connected,check_clients_messages).chain());
        app.add_systems(Last,(check_connection_up,start_listening_clients,restart_connection).chain());
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

pub fn check_client_connections_down(
    mut server_connections: ResMut<ServerConnections>,
    mut client_diconnected: MessageWriter<ClientDiconnected>,
){
    for (_,connection) in server_connections.0.iter_mut() {
        match connection {
            ServerConnectionType::Tcp(server_connection) => {
                let mut remove_list: Vec<Uuid> = Vec::new();

                for (uuid,client_connection) in server_connection.connections.iter_mut()  {
                    match client_connection.connection_down_receiver.try_recv() {
                        Ok(_) => {
                            client_connection.listening = false;

                            client_diconnected.write(ClientDiconnected(*uuid, ConnectionsType::Tcp, server_connection.name));

                            remove_list.push(*uuid);
                        }
                        Err(_) => {

                        }
                    }
                }

                for uuid in remove_list {
                    server_connection.connections.remove(&uuid);
                }
            }
        }
    }
}

pub fn check_clients_connected(
    mut server_connections: ResMut<ServerConnections>,
    mut client_connected_event: MessageWriter<ClientConnected>,
    mut new_clients_to_replicate: Option<ResMut<NewClientsToReplicate>>,
){
    for (_,connection) in server_connections.0.iter_mut() {
        match connection {
            ServerConnectionType::Tcp(connection) => {
                match connection.client_connected_receiver.try_recv() {
                    Ok((tcp_stream,_)) => {
                        let settings = &connection.settings;
                        let mut tcp_connection = TcpConnection::new(tcp_stream, connection.name, NetworkSide::Server, Arc::clone(&connection.cancel_token),settings.bytes,settings.order);
                        let current_uuid = tcp_connection.uuid.unwrap();

                        client_connected_event.write(ClientConnected(current_uuid,ConnectionsType::Tcp,connection.name));
                        
                        tcp_connection.send_message(&ConnectedMessage{
                            uuid: current_uuid
                        },connection.runtime.as_ref().unwrap());
                        
                        connection.connections.insert(current_uuid,tcp_connection);
                        
                        if let Some(new_clients_to_replicate) = new_clients_to_replicate.as_mut() {
                            new_clients_to_replicate.0.push(current_uuid);
                        }
                    }
                    Err(_) => {
                        continue
                    }
                }
            }
        }
    }
}

pub fn check_clients_messages(
    mut server_connections: ResMut<ServerConnections>,
    mut commands: Commands,
){
    for (_,connection) in server_connections.0.iter_mut() {
        match connection {
            ServerConnectionType::Tcp(connection) => {
                for (uuid,client_connection) in connection.connections.iter_mut()  {
                    match client_connection.message_received_receiver.try_recv() {
                        Ok(message) => {
                            let connection_name = connection.name;
                            let uuid = *uuid;

                            commands.queue(move |w: &mut World| {
                                let type_id = message.as_any().type_id();
                                let map = DISPATCHERS.lock().unwrap();
                                if let Some(dispatcher) = map.get(&type_id) {
                                    let boxed_any = message as Box<dyn Any>;

                                    dispatcher(boxed_any, w, ConnectionsType::Tcp, Some(uuid), &NetworkSide::Server, connection_name);
                                } else {
                                    println!("This message does not exist");
                                }
                            });
                        }
                        Err(_) => {

                        }
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

pub fn start_listening_clients(
    mut server_connections: ResMut<ServerConnections>,
){
    for (_,connection) in server_connections.0.iter_mut() {
        match connection {
            ServerConnectionType::Tcp(connection) => {
                for (_,client_connection) in connection.connections.iter_mut()  {
                    if client_connection.listening {continue}

                    client_connection.start_listening(&connection.runtime.as_ref().unwrap());
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