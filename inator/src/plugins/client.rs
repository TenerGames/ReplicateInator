use bevy::app::App;
use bevy::prelude::{First, Last, Plugin, ResMut};
use crate::connections::{ClientConnectionType, ClientConnections, Connection, Connections};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientConnections::new());
        app.add_systems(First,start_connections);
        app.add_systems(Last,restart_connection);
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

pub fn restart_connection(
    mut client_connections: ResMut<ClientConnections>,
){
    for (_,connection) in client_connections.0.iter_mut() {
        match connection {
            ClientConnectionType::Tcp(connection) => {
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