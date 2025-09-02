use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::prelude::{IntoScheduleConfigs, Startup};
use inator::plugins::client::ClientPlugin;
use inator::plugins::server::ServerPlugin;

mod client;
mod server;

fn main() {
    App::new().add_plugins((DefaultPlugins,ServerPlugin,ClientPlugin))
        .add_systems(Startup,(server::create_connection,client::create_connection).chain())
        .run();
}
