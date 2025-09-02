use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Mutex;
use bevy::app::App;
use bevy::prelude::{Event, World};
use bincode::config::standard;
use typetag::__private::once_cell::sync::Lazy;
use uuid::Uuid;
use crate::connections::ConnectionsType;
use crate::NetworkSide;

pub struct MessagingPlugin;

#[typetag::serde]
pub trait MessageTrait: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Event)]
pub struct MessageReceivedFromServer<T: MessageTrait>{
    pub message: T,
    pub message_type: ConnectionsType,
    pub connection_name: &'static str
}

#[derive(Event)]
pub struct MessageReceivedFromClient<T: MessageTrait>{
    pub message: T,
    pub message_type: ConnectionsType,
    pub sender: Option<Uuid>,
    pub connection_name: &'static str
}

pub type DispatcherFn = Box<dyn Fn(Box<dyn Any>, &mut World, ConnectionsType, Option<Uuid>, &NetworkSide, &'static str) + Send + Sync>;

pub static DISPATCHERS: Lazy<Mutex<HashMap<TypeId, DispatcherFn>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[macro_export]
macro_rules! register_message_type {
    ($type:ty, $dispatcher_map:expr) => {{
        use std::any::TypeId;

        let dispatcher: DispatcherFn = Box::new(
            |boxed, world, message_type, uuid, network_side, connection_name| {
                let msg = boxed.downcast::<$type>().expect("Failed to downcast");

                if network_side == &NetworkSide::Client {
                    world.send_event(MessageReceivedFromServer {
                        message: *msg,
                        message_type,
                        connection_name,
                    });
                } else {
                    world.send_event(MessageReceivedFromClient {
                        message: *msg,
                        message_type,
                        sender: uuid,
                        connection_name,
                    });
                }
            },
        );

        let mut map = $dispatcher_map.lock().unwrap();
        let type_id = TypeId::of::<$type>();

        if !map.contains_key(&type_id) {
           map.insert(type_id, dispatcher);
        }
    }};
}

pub fn deserialize_message(buf: &[u8]) -> Option<Box<dyn MessageTrait>> {
    let config = standard();
    let mut cursor = Cursor::new(buf);

    match bincode::serde::decode_from_std_read::<Box<dyn MessageTrait>, _, _>(&mut cursor, config) {
        Ok(msg) => Some(msg),
        Err(e) => {println!("Decode error {} ", e); None},
    }
}

pub fn register_message_type<T: MessageTrait>(app: &mut App, network_side: &NetworkSide){
    if network_side == &NetworkSide::Client {
        app.add_event::<MessageReceivedFromServer<T>>();
    }else {
        app.add_event::<MessageReceivedFromClient<T>>();
    }

    register_message_type!(T, &DISPATCHERS);
}