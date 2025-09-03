use crate::systems::messaging::{register_message_type, MessageReceivedFromServer, MessageTrait};
use std::any::{TypeId};
use std::collections::HashMap;
use bevy::prelude::{Added, App, Changed, Commands, Component, Entity, EventReader, Plugin, Query, Reflect, Res, ResMut, Resource, Update};
use bevy::reflect::{GetTypeRegistration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use message_derive::Message;
use crate::connections::{ClientConnections, ServerConnectionType, ServerConnections};
use crate::NetworkSide;

pub trait RegisterReplicatedComponent{
    fn register_replicated_component<T: Component + Reflect + Clone + GetTypeRegistration + Serialize>(&mut self);
}

pub struct ReplicatingPlugin {
    pub network_side: NetworkSide
}

#[derive(Component)]
pub struct Replicated{
    pub(crate) connection_name: &'static str,
    pub(crate) owner: Option<Uuid>,
    pub(crate) black_list_components: Vec<TypeId>,
    pub(crate) entity_ref: Option<Entity>,
}

#[derive(Serialize, Deserialize, Message)]
pub struct ReplicatedMessageServer {
    pub entity_ref: Entity,
    pub component_name: String,
    pub component_data: Vec<u8>,
    pub first_replication: bool
}

#[derive(Serialize, Deserialize, Message)]
pub struct ReplicatedMessageClient {
    pub uuid: Uuid,
    pub entity_ref: Entity,
}

#[derive(Default, Resource)]
pub struct ReplicateRegistry(HashMap<String, TypeId>);

impl RegisterReplicatedComponent for App{
    fn register_replicated_component<T: Component + Reflect + Clone + GetTypeRegistration + Serialize>(&mut self) {
        self.register_type::<T>();

        let mut registry = self.world_mut().resource_mut::<ReplicateRegistry>();

        registry.0.insert(std::any::type_name::<T>().parse().unwrap(),TypeId::of::<T>());

        self.add_systems(Update,detect_components_changed::<T>);
    }
}

impl Replicated {
    pub fn new(connection_name: &'static str, owner: Option<Uuid>, black_list_components: Vec<TypeId>) -> Self {
        Self{
            connection_name,
            owner,
            black_list_components,
            entity_ref: None
        }
    }
}

impl Plugin for ReplicatingPlugin{
    fn build(&self, app: &mut App) {
        app.insert_resource(ReplicateRegistry::default());

        if self.network_side == NetworkSide::Client {
            register_message_type::<ReplicatedMessageServer>(app, &self.network_side);
            app.add_systems(Update,client_check_server_replications);
        }else{
            register_message_type::<ReplicatedMessageClient>(app, &self.network_side);
        }
    }
}

pub fn detect_components_changed<T: Component + Reflect + Clone + Serialize>(
    changed_query: Query<(Entity, &T, &Replicated), Changed<T>>,
    added_query: Query<(Entity, &T, &Replicated), Added<Replicated>>,
    mut client_connections: Option<ResMut<ClientConnections>>,
    mut server_connections: Option<ResMut<ServerConnections>>,
){
    for (entity, component, replicated) in added_query.iter() {
        let connection_name = replicated.connection_name;

        if let Some(_client_connections) = client_connections.as_mut() {

        }

        if let Some(server_connections) = server_connections.as_mut() {
            let connection = server_connections.0.get_mut(connection_name);

            if let Some(connection) = connection {
                match connection {
                    ServerConnectionType::Tcp(tcp_connection) => {
                        for (_, client_connection) in tcp_connection.connections.iter_mut() {
                            client_connection.replicate_entity(entity, tcp_connection.runtime.as_ref().unwrap(), component.clone(), true);
                        }
                    }
                }
            }
        }
    }
}

pub fn client_check_server_replications(
    mut replicated_message_server_event: EventReader<MessageReceivedFromServer<ReplicatedMessageServer>>,
    mut commands: Commands,
    replicate_registry: Res<ReplicateRegistry>,
) {
    for event in replicated_message_server_event.read() {
        let message = &event.message;

        if message.first_replication {
            let entity = commands.spawn(Replicated {
                connection_name: event.connection_name,
                owner: None,
                black_list_components: vec![],
                entity_ref: Some(message.entity_ref),
            }).id();

            let type_id = replicate_registry.0.get(&message.component_name);


        }
    }
}

