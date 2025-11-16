use bevy::app::{App, Startup, Update};
use bevy::asset::uuid;
use bevy::DefaultPlugins;
use bevy::prelude::{Added, Commands, Entity, IntoScheduleConfigs, Query, ResMut};
use inator::connections::{ServerConnections};
use inator::connections::tcp::server::ServerTcpSettings;
use inator::NetworkSide;
use inator::plugins::replication::{Replicated, ReplicatingPlugin};
use inator::plugins::server::ServerPlugin;
use shared::{Health, SharedPlugin};

pub fn create_connection(
    mut server_connections: ResMut<ServerConnections>,
){
    server_connections.new_server_tcp_connection(ServerTcpSettings::default(),"Lobby");
}

pub fn start_test(
    mut commands: Commands,
){
    commands.spawn((
        Health{value:10},
        Replicated{
            connection_name: "Lobby".to_string(),
            entity_ref: uuid::Uuid::new_v4().into_bytes(),
        }
    ));
}

pub fn test_health(
    health_query: Query<(Entity, &Health), Added<Health>>,
){
    for (_, health) in health_query.iter() {
        println!("Health on server is: {:?}", health.value);
    }
}

fn main() {
    App::new().add_plugins((DefaultPlugins,ServerPlugin, ReplicatingPlugin{
        network_side: NetworkSide::Server,
    },SharedPlugin{
        network_side: NetworkSide::Server,
    }))
        .add_systems(Startup,(create_connection,start_test).chain())
        .add_systems(Update, test_health)
        .run();
}
