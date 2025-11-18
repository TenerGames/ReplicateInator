use crate::systems::messaging::MessageTrait;
use serde::{Deserialize, Serialize} ;
use uuid::Uuid;
use message_derive::Message;
use bevy::prelude::{Message as BevyMessage};
use crate::connections::ConnectionsType;

pub mod client;
pub mod server;
pub mod replication;

#[derive(BevyMessage)]
pub struct ClientConnected(pub Uuid, pub ConnectionsType, pub &'static str);

#[derive(BevyMessage)]
pub struct ClientDiconnected(pub Uuid, pub ConnectionsType, pub &'static str);

#[derive(Serialize, Deserialize, Message)]
pub(crate) struct ConnectedMessage {
    pub uuid: Uuid
}

