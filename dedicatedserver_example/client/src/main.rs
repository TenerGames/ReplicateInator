use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::ResMut;
use inator::connections::{ClientConnections, Connections};
use inator::connections::tcp::client::ClientTcpSettings;
use inator::plugins::client::ClientPlugin;

pub fn create_connection(
    mut client_connections: ResMut<ClientConnections>,
){
    client_connections.new_client_tcp_connection(ClientTcpSettings::default(),"Lobby");
}

fn main() {
    App::new().add_plugins((DefaultPlugins,ClientPlugin))
        .add_systems(Startup,create_connection)
        .run();
}
