use bevy::app::{App, Plugin};
use crate::connections::{ClientConnections, Connections, ServerConnections};
use crate::NetworkSide;

pub struct MessagingPlugin {
    pub network_side: NetworkSide
}

impl Plugin for MessagingPlugin {
    fn build(&self, app: &mut App) {
        match self.network_side{
            NetworkSide::Client => app.insert_resource(ClientConnections::new()),
            NetworkSide::Server => app.insert_resource(ServerConnections::new()),
            _ => unreachable!()
        };
    }
}
