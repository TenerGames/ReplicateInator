use crate::systems::messaging::MessageTrait;
use std::net::SocketAddr;
use bevy::prelude::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use message_derive::Message;

pub mod client;
pub mod server;

#[derive(Event)]
pub struct ClientConnected(pub SocketAddr, pub &'static str);

#[derive(Serialize, Deserialize, Message)]
pub struct ConnectedMessage {
    pub uuid: Uuid
}

