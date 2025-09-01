pub mod connections;
pub mod plugins;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Eq,PartialEq)]
pub enum NetworkSide{
    Client,
    Server
}

#[cfg(test)]
mod tests {
    use bevy::{MinimalPlugins};
    use bevy::prelude::App;
    use crate::plugins::client::ClientPlugin;
    use crate::plugins::server::ServerPlugin;

    #[test]
    fn it_works() {
        //Testing server and client

        App::new().add_plugins((MinimalPlugins,ServerPlugin)).run();

        App::new().add_plugins((MinimalPlugins,ClientPlugin)).run();
    }
}
