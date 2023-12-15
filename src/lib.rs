use std::{
    any::{Any, TypeId},
};

mod context;
pub use self::context::Context;

mod handle;
pub use self::handle::Handle;

pub mod signal;
pub use self::signal::Signal;

pub trait Object{
    fn start(self) -> Handle<Self> where Self: Sized + 'static {
        Handle::new(self)
    }
}

struct ListenerData {
    msg_id: TypeId,
    listener_id: TypeId,
    f: Box<dyn FnMut(&dyn Any)>,
}

struct Node {
    object: Box<dyn Any>,
    listeners: Vec<ListenerData>,
}
