use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::{Added, Entity, Query, ResMut, Update};
use inator::connections::{ClientConnections};
use inator::connections::tcp::client::ClientTcpSettings;
use inator::NetworkSide;
use inator::plugins::client::ClientPlugin;
use inator::plugins::replication::ReplicatingPlugin;
use shared::{Health, SharedPlugin};

pub fn create_connection(
    mut client_connections: ResMut<ClientConnections>,
){
    client_connections.new_client_tcp_connection(ClientTcpSettings::default(),"Lobby");
}

pub fn test_health(
    health_query: Query<(Entity, &Health), Added<Health>>,
){
    for (_, health) in health_query.iter() {
        println!("Health on client is: {:?}", health.value);
        println!("Health server_only_value on client is: {:?}", health.server_only_value);
    }
}

fn main() {
    App::new().add_plugins((DefaultPlugins,ClientPlugin,ReplicatingPlugin{
        network_side: NetworkSide::Client
    },SharedPlugin{
        network_side: NetworkSide::Client
    }))
        .add_systems(Startup,create_connection)
        .add_systems(Update, test_health)
        .run();
}
