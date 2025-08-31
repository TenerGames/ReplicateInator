use bevy::app::{App, Plugin};
use crate::connections::{ClientConnections, Connections, ServerConnections};
use crate::NetworkSide;

pub struct MessagingPlugin {
    pub network_side: NetworkSide
}

impl Plugin for MessagingPlugin {
    fn build(&self, app: &mut App) {
        if self.network_side == NetworkSide::Client {
            app.insert_resource(ClientConnections::new());
        }else{
            app.insert_resource(ServerConnections::new());
        }
    }
}