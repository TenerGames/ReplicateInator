use bevy::prelude::ResMut;
use inator::connections::{Connections, ServerConnections};

use inator::connections::tcp::server::ServerTcpSettings;

pub fn create_connection(
    mut server_connections: ResMut<ServerConnections>,
){
    server_connections.new_server_tcp_connection(ServerTcpSettings::default(),"Lobby");
}