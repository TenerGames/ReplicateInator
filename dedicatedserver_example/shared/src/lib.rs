use inator::plugins::replication::ComponentReplicated;
use bevy::app::App;
use bevy::prelude::{Plugin, Reflect};
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use component_replicated_derive::component_replicated;
use inator::{NetworkSide};
use inator::plugins::replication::{RegisterReplicatedComponent};

pub struct SharedPlugin{
    pub network_side: NetworkSide,
}

#[component_replicated]
pub struct Health {
    pub value: u32,

    #[dont_replicate]
    pub server_only_value: bool,
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    name: String,
    #[inspector(min = 0.0, max = 1.0)]
    option: f32,
}

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Configuration>();
        app.register_type::<Configuration>();
        app.add_plugins(EguiPlugin::default());
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_plugins(ResourceInspectorPlugin::<Configuration>::default());
        app.add_plugins(ResourceInspectorPlugin::<Time>::default());
        app.register_replicated_component::<Health>(&self.network_side);
    }
}
