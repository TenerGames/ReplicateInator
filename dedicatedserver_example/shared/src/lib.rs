use bincode::{Decode, Encode};
use bevy::prelude::ReflectComponent;
use bevy::app::App;
use bevy::prelude::{Component, Plugin, Reflect};
use bevy::reflect::erased_serde::__private::serde::{Deserialize, Serialize};
use inator::NetworkSide;
use inator::plugins::replication::{ComponentReplicated, RegisterReplicatedComponent};

pub struct SharedPlugin{
    pub network_side: NetworkSide,
}

#[derive(Component, Serialize, Deserialize, Clone, Reflect, Decode, Encode)]
#[reflect(Component)]
pub struct Health{
    pub value: u32,
}


impl ComponentReplicated for Health{

}

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.register_replicated_component::<Health>(&self.network_side);
    }
}
