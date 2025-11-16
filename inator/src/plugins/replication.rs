use std::any::TypeId;
use std::collections::HashMap;
use bevy::app::App;
use bevy::log::error;
use bevy::prelude::{Added, AppTypeRegistry, Changed, Commands, Component, Entity, EventReader, Last, ParamSet, Plugin, PostUpdate, Query, Reflect, ReflectComponent, Res, ResMut, Resource, Update, With, Without, World};
use bevy::reflect::GetTypeRegistration;
use bincode::config::standard;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use message_derive::Message;
use crate::connections::{ServerConnections};
use crate::NetworkSide;
use crate::systems::messaging::{register_message_type, MessageReceivedFromServer, MessageTrait};

pub struct ReplicatingPlugin {
    pub network_side: NetworkSide
}
pub struct ReplicationInfo{
    type_id: TypeId,
    deserialize_fn: fn(&[u8]) -> Box<dyn Reflect>,
}

#[derive(Component)]
pub struct FirstReplicated;

#[derive(Serialize, Deserialize, Message)]
pub struct ReplicateMessageFromServer{
    replicated_byes: Vec<u8>,
    components: HashMap<i32,Vec<u8>>
}

#[derive(Component, Decode, Encode)]
pub struct Replicated{
    pub connection_name: String,
    pub entity_ref: [u8; 16]
}

pub struct ReplicateTo{
    all_clients: bool,
    to_clients: Vec<Uuid>,
    bytes: HashMap<i32, Vec<u8>>,
}

#[derive(Default,Resource)]
pub struct ReplicationComponentsRegistry(i32, HashMap<TypeId, i32>, HashMap<i32, ReplicationInfo>);
#[derive(Default,Resource)]
pub struct ServerReplicationQueue(HashMap<Entity, ReplicateTo>);
#[derive(Default,Resource)]
pub struct ReplicatedEntities(HashMap<Uuid, Entity>);
#[derive(Default,Resource)]
pub struct NewClientsToReplicate(pub(crate) Vec<Uuid>);

pub trait ComponentReplicated: Component + GetTypeRegistration + Reflect + Decode<()> + Encode {}

pub trait RegisterReplicatedComponent{
    fn register_replicated_component<T: ComponentReplicated>(&mut self, network_side: &NetworkSide) -> &mut Self;
}

pub fn component_changed_server<T: ComponentReplicated>(
    added_query: Query<(Entity, &mut Replicated, &T), (Added<Replicated>, Without<FirstReplicated>)>,
    mut set: ParamSet<(
        Query<(Entity, &mut Replicated, &T), (With<Replicated>, With<FirstReplicated>)>,
        Query<(Entity, &mut Replicated, &T), (With<Replicated>, Changed<T>, With<FirstReplicated>)>,
    )>,
    replication_components_registry: Res<ReplicationComponentsRegistry>,
    mut server_components_queue: ResMut<ServerReplicationQueue>,
    new_clients_to_replicate: Res<NewClientsToReplicate>,
    mut commands: Commands
){
    let type_id = TypeId::of::<T>();
    let id_registry = replication_components_registry.1.get(&type_id).unwrap();
    let config = standard();
    let mut updated = false;

    for (entity, _, comp) in &added_query {
        let replicate_to = server_components_queue.0.get_mut(&entity);

        if let Some(replicate_to) = replicate_to {
            replicate_to.bytes.insert(*id_registry, bincode::encode_to_vec(comp, config).unwrap());
        }else{
            server_components_queue.0.insert(entity,ReplicateTo{
                all_clients: true,
                to_clients: Vec::new(),
                bytes: HashMap::from([
                    (*id_registry, bincode::encode_to_vec(comp, config).unwrap())
                ]),
            });
        }

        commands.entity(entity).insert(FirstReplicated);

        updated = true;
    }

    if updated {return;}

    if new_clients_to_replicate.0.len() > 0 {
        for (entity, _, comp) in set.p0().iter() {
            let replicate_to = server_components_queue.0.get_mut(&entity);

            if let Some(replicate_to) = replicate_to {
                replicate_to.bytes.insert(*id_registry, bincode::encode_to_vec(comp, config).unwrap());

                for client in &new_clients_to_replicate.0 {
                    replicate_to.to_clients.push(*client);
                }
            }else{
                let mut replicate_to_new = ReplicateTo{
                    all_clients: false,
                    to_clients: vec![],
                    bytes: HashMap::from([
                        (*id_registry, bincode::encode_to_vec(comp, config).unwrap())
                    ]),
                };

                for client in &new_clients_to_replicate.0 {
                    replicate_to_new.to_clients.push(*client);
                }

                server_components_queue.0.insert(entity,replicate_to_new);
            }
        }

        updated = true;
    }

    if updated {return;}

    for (entity, _, comp) in set.p1().iter() {
        let replicate_to = server_components_queue.0.get_mut(&entity);

        if let Some(replicate_to) = replicate_to {
            replicate_to.bytes.insert(*id_registry, bincode::encode_to_vec(comp, config).unwrap());
        }else{
            server_components_queue.0.insert(entity,ReplicateTo{
                all_clients: true,
                to_clients: Vec::new(),
                bytes: HashMap::from([
                    (*id_registry, bincode::encode_to_vec(comp, config).unwrap())
                ]),
            });
        }

        commands.entity(entity).insert(FirstReplicated);
    }
}

pub fn deserialize_component<T: ComponentReplicated>(bytes: &[u8]) -> Box<dyn Reflect> {
    let (val, _): (T, usize) =
        bincode::decode_from_slice(bytes, standard()).unwrap();

    Box::new(val)
}

impl ReplicationComponentsRegistry {
    pub fn registry<T: ComponentReplicated>(&mut self) {
        let type_id = TypeId::of::<T>();

        if self.is_registered(&type_id) { return; }

        let new_id = self.0 + 1;

        self.0 = new_id;
        self.1.insert(type_id, new_id);
        self.2.insert(new_id, ReplicationInfo{
            type_id,
            deserialize_fn: deserialize_component::<T>,
        });
    }

    pub fn is_registered(&self, type_id: &TypeId) -> bool {
        self.1.contains_key(type_id)
    }
}

impl RegisterReplicatedComponent for App{
    fn register_replicated_component<T: ComponentReplicated>(&mut self, network_side: &NetworkSide) -> &mut Self {
        self.register_type::<T>();

        let word_mut = self.world_mut();
        let mut replication_components_registry = match word_mut.get_resource_mut::<ReplicationComponentsRegistry>() {
            None => {
                error!("ReplicationComponentsRegistry was not registered");
                return self;
            }
            Some(replication_components_registry) => {
                replication_components_registry
            }
        };

        replication_components_registry.registry::<T>();

        if network_side == &NetworkSide::Server {
            self.add_systems(Update,component_changed_server::<T>);
        }else if network_side == &NetworkSide::Client {

        }else if network_side == &NetworkSide::LocalServer {
            self.add_systems(Update,component_changed_server::<T>);
        }

        self
    }
}

impl Plugin for ReplicatingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ReplicationComponentsRegistry::default());
        app.insert_resource(ReplicatedEntities::default());

        if self.network_side == NetworkSide::Server {
            app.insert_resource(ServerReplicationQueue::default());
            app.insert_resource(NewClientsToReplicate::default());

            app.add_systems(PostUpdate,replicate_to_client);
        }else if self.network_side == NetworkSide::Client {
            app.add_systems(Last,replication_from_server);

            register_message_type::<ReplicateMessageFromServer>(app, &NetworkSide::Client);
        }else if self.network_side == NetworkSide::LocalServer {
            app.insert_resource(ServerReplicationQueue::default());
            app.insert_resource(NewClientsToReplicate::default());

            register_message_type::<ReplicateMessageFromServer>(app, &NetworkSide::Client);

            app.add_systems(PostUpdate,replicate_to_client);
            app.add_systems(Last,replication_from_server);
        }
    }
}

pub fn replicate_to_client(
    replicate_query : Query<(Entity, &Replicated), With<Replicated>>,
    mut server_components_queue: ResMut<ServerReplicationQueue>,
    mut server_connections: ResMut<ServerConnections>,
    mut new_clients_to_replicate: ResMut<NewClientsToReplicate>,
){
    let config = standard();

    for (entity, replicated) in replicate_query {
        if server_components_queue.0.contains_key(&entity) {
            let replicate_to = server_components_queue.0.remove(&entity).unwrap();
            let string_ref: String = replicated.connection_name.parse().unwrap();

            if replicate_to.all_clients{
                server_connections.send_for_all_clients(&ReplicateMessageFromServer{
                    replicated_byes: bincode::encode_to_vec(replicated, config).unwrap(),
                    components: replicate_to.bytes,
                }, &string_ref);
            }else{
                server_connections.send_to_clients(&ReplicateMessageFromServer{
                    replicated_byes: bincode::encode_to_vec(replicated, config).unwrap(),
                    components: replicate_to.bytes,
                }, &string_ref, &replicate_to.to_clients)
            }

            for uuid in replicate_to.to_clients{
                new_clients_to_replicate.0.retain(|u| *u != uuid);
            }
        }
    }
}

pub fn replication_from_server(
    mut replicate_message_from_server: EventReader<MessageReceivedFromServer::<ReplicateMessageFromServer>>,
    mut replicated_entities: ResMut<ReplicatedEntities>,
    replication_components_registry: Res<ReplicationComponentsRegistry>,
    mut commands: Commands
){
    let config = standard();

    for ev in replicate_message_from_server.read() {
        let message = &ev.message;
        let components_bytes = &message.components;
        let (replicated, _): (Replicated, usize) = bincode::decode_from_slice(&*message.replicated_byes, config).unwrap();
        let entity_ref = Uuid::from_bytes(replicated.entity_ref);
        let have_entity = replicated_entities.0.get(&entity_ref);

        if let Some(entity) = have_entity {
            for (registry_id, bytes) in components_bytes {
                let replication_infos = replication_components_registry
                    .2
                    .get(registry_id)
                    .unwrap();

                let reflected_value = (replication_infos.deserialize_fn)(bytes);

                let type_id = replication_infos.type_id;
                let reflected_value = reflected_value; // Box<dyn Reflect>
                let entity_id = *entity;

                commands.queue(move |world: &mut World| {
                    world.resource_scope::<AppTypeRegistry, _>(|world, app_registry| {
                        let entity = world.entity_mut(entity_id);

                        let registry = app_registry.read();

                        let type_reg = registry
                            .get(type_id)
                            .expect("Tipo não registrado no TypeRegistry!");

                        let reflect_component = type_reg
                            .data::<ReflectComponent>()
                            .expect("Tipo não é ReflectComponent!");

                        reflect_component.apply(entity,reflected_value.as_ref());
                    });
                });
            }
        }else{
            let entity = commands.spawn((
                Replicated{
                    connection_name: replicated.connection_name,
                    entity_ref: replicated.entity_ref,
                },
                FirstReplicated
            )
            ).id();
            replicated_entities.0.insert(entity_ref, entity);

            for (registry_id, bytes) in components_bytes {
                let replication_infos = replication_components_registry
                    .2
                    .get(registry_id)
                    .unwrap();

                let reflected_value = (replication_infos.deserialize_fn)(bytes);

                let type_id = replication_infos.type_id;
                let reflected_value = reflected_value; // Box<dyn Reflect>
                let entity_id = entity;

                commands.queue(move |world: &mut World| {
                    world.resource_scope::<AppTypeRegistry, _>(|world, app_registry| {
                        let mut entity = world.entity_mut(entity_id);

                        let registry = app_registry.read();

                        let type_reg = registry
                            .get(type_id)
                            .expect("Tipo não registrado no TypeRegistry!");

                        let reflect_component = type_reg
                            .data::<ReflectComponent>()
                            .expect("Tipo não é ReflectComponent!");

                        reflect_component.insert(
                            &mut entity,
                            reflected_value.as_ref(),
                            &registry
                        );
                    });
                });
            }
        }
    }
}