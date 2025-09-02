use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::ResMut;
use inator::connections::{Connections, ServerConnections};
use inator::connections::tcp::server::ServerTcpSettings;
use inator::plugins::server::ServerPlugin;

pub fn create_connection(
    mut server_connections: ResMut<ServerConnections>,
){
    server_connections.new_server_tcp_connection(ServerTcpSettings::default(),"Lobby");
}

fn main() {
    App::new().add_plugins((DefaultPlugins,ServerPlugin))
        .add_systems(Startup,create_connection)
        .run();
}
