use std::net::SocketAddr;
use bevy::prelude::Event;


pub mod client;
pub mod server;

#[derive(Event)]
pub struct ClientConnected(pub SocketAddr, pub &'static str);