use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::{ResMut};
use inator::connections::{ClientConnections};
use inator::connections::tcp::client::ClientTcpSettings;
use inator::NetworkSide;
use inator::plugins::client::ClientPlugin;
use inator::plugins::replication::ReplicatingPlugin;
use shared::SharedPlugin;

pub fn create_connection(
    mut client_connections: ResMut<ClientConnections>,
){
    client_connections.new_client_tcp_connection(ClientTcpSettings::default(),"Lobby");
}

fn main() {
    App::new().add_plugins((DefaultPlugins,ClientPlugin,ReplicatingPlugin{
        network_side: NetworkSide::Client
    },SharedPlugin{
        network_side: NetworkSide::Client
    }))
        .add_systems(Startup,create_connection)
        .run();
}
