use crate::systems::messaging::MessageTrait;
use bevy::prelude::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use message_derive::Message;
use crate::connections::ConnectionsType;

pub mod client;
pub mod server;
pub mod replication;

#[derive(Event)]
pub struct ClientConnected(pub Uuid, pub ConnectionsType, pub &'static str);

#[derive(Event)]
pub struct ClientDiconnected(pub Uuid, pub ConnectionsType, pub &'static str);

#[derive(Serialize, Deserialize, Message)]
pub struct ConnectedMessage {
    pub uuid: Uuid
}

