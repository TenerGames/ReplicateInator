use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Mutex;
use bevy::app::App;
use bevy::prelude::{Event, World};
use bincode::config::standard;
use typetag::__private::once_cell::sync::Lazy;

pub struct MessagingPlugin;

pub enum MessageType{
    Tcp
}

#[typetag::serde]
pub trait MessageTrait: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Event)]
pub struct MessageReceived<T: MessageTrait>{
    pub message: T,
    pub message_type: MessageType,
}

pub type DispatcherFn = Box<dyn Fn(Box<dyn Any>, &mut World, MessageType) + Send + Sync>;

pub static DISPATCHERS: Lazy<Mutex<HashMap<TypeId, DispatcherFn>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[macro_export]
macro_rules! register_message_type {
    ($type:ty, $dispatcher_map:expr) => {{
        use std::any::TypeId;
        let dispatcher: DispatcherFn = Box::new(|boxed, world, message_type| {
            let msg = boxed.downcast::<$type>().expect("Failed to downcast");
            world.send_event(MessageReceived {
                message: *msg,
                message_type: message_type,
            });
        });
        $dispatcher_map.lock().unwrap().insert(TypeId::of::<$type>(), dispatcher);
    }};
}

pub fn deserialize_message(buf: &[u8]) -> Option<Box<dyn MessageTrait>> {
    let config = standard();
    let mut cursor = Cursor::new(buf);

    match bincode::serde::decode_from_std_read::<Box<dyn MessageTrait>, _, _>(&mut cursor, config) {
        Ok(msg) => Some(msg),
        Err(_) => None,
    }
}

pub fn register_message_type<T: MessageTrait>(app: &mut App){
    app.add_event::<MessageReceived<T>>();
    register_message_type!(T, &DISPATCHERS);
}