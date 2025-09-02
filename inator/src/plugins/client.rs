use std::any::Any;
use std::sync::Arc;
use bevy::app::App;
use bevy::prelude::{Commands, First, IntoScheduleConfigs, Last, Plugin, ResMut, Update, World};
use crate::connections::{ClientConnectionType, ClientConnections, Connection, Connections, ConnectionsType};
use crate::connections::tcp::connection::TcpConnection;
use crate::NetworkSide;
use crate::plugins::ConnectedMessage;
use crate::systems::messaging::{register_message_type, DISPATCHERS};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        register_message_type::<ConnectedMessage>(app, &NetworkSide::Client);

        app.insert_resource(ClientConnections::new());
        app.add_systems(First,start_connections);
        app.add_systems(Update,check_new_messages);
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

pub fn check_new_messages(
    mut client_connections: ResMut<ClientConnections>,
    mut commands: Commands,
){
    for (_,connection) in client_connections.0.iter_mut() {
        match connection {
            ClientConnectionType::Tcp(connection) => {
                match connection.local_tcp_connection.as_mut() {
                    Some(local_tcp_connection) => {
                        match local_tcp_connection.message_received_receiver.try_recv() {
                            Ok(message) => {
                                let connection_name = connection.name;
                                
                                if let Some(connected_message) = message.as_any().downcast_ref::<ConnectedMessage>() {
                                    println!("connected message");
                                    local_tcp_connection.uuid = Some(connected_message.uuid);
                                }
                                
                                let uuid = local_tcp_connection.uuid;
                                
                                commands.queue(move |w: &mut World| {
                                    let type_id = message.as_any().type_id();
                                    let map = DISPATCHERS.lock().unwrap();
                                    if let Some(dispatcher) = map.get(&type_id) {
                                        let boxed_any = message as Box<dyn Any>;

                                        dispatcher(boxed_any, w, ConnectionsType::Tcp, uuid, &NetworkSide::Client, connection_name);
                                    } else {
                                        println!("This message does not exist");
                                    }
                                });
                            },
                            Err(_) => continue,
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

                        let settings = &connection.settings;
                        let mut tcp_connection = TcpConnection::new(tcp_stream, connection.name, NetworkSide::Client, Arc::clone(&connection.cancel_token),settings.bytes,settings.order);
                        
                        tcp_connection.start_listening(connection.runtime.as_ref().unwrap());

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