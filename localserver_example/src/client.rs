use bevy::prelude::{ResMut};
use inator::connections::{ClientConnections, Connections};
use inator::connections::tcp::client::ClientTcpSettings;

pub fn create_connection(
    mut client_connections: ResMut<ClientConnections>,
){
    client_connections.new_client_tcp_connection(ClientTcpSettings::default(),"Lobby");
}