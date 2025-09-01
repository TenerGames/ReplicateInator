use bevy::app::App;
use bevy::prelude::{First, IntoScheduleConfigs, Last, Plugin, ResMut};
use crate::connections::{ClientConnectionType, ClientConnections, Connection, Connections};

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
                    Ok((read_half, write_half)) => {
                        if let Some(read_half) = connection.read_half.take() {
                            drop(read_half);
                        }

                        if let Some(write_half) = connection.write_half.take() {
                            drop(write_half);
                        }
                        
                        connection.read_half = Some(read_half);
                        connection.write_half = Some(write_half);
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