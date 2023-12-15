use std::{
    any::{Any, TypeId},
    cell::RefCell,
    rc::Rc,
};

mod context;
pub use self::context::Context;

mod handle;
pub use self::handle::Handle;

pub trait Object {}

pub fn emit<M: 'static>(cx: &mut Context<impl Signal<M>>, msg: M) {
    for listener in &mut cx.node.listeners {
        if listener.msg_type_id == msg.type_id() {
            (listener.f)(&msg)
        }
    }
}

pub trait Signal<M: 'static>: Sized {
    fn emit(cx: &mut Context<Self>, msg: M) {
        emit(cx, msg)
    }
}

struct ListenerData {
    msg_type_id: TypeId,
    listener_type_id: TypeId,
    f: Box<dyn FnMut(&dyn Any)>,
}

struct Node {
    object: Box<dyn Any>,
    listeners: Vec<ListenerData>,
}

pub struct Listener {
    type_id: TypeId,
    node: Rc<RefCell<Node>>,
}

impl Listener {
    pub fn unlisten(&self) -> bool {
        let mut node = self.node.borrow_mut();
        if let Some(idx) = node
            .listeners
            .iter()
            .position(|listener| listener.listener_type_id == self.type_id)
        {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }
}
