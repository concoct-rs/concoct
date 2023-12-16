//! Signals

use crate::Context;
use core::{any::Any, mem};

/// The default implementation of `Signal::emit`.
pub fn emit<M: 'static>(cx: &mut Context<impl Signal<M>>, msg: M) {
    let listeners = cx.node.as_ref().unwrap().listeners.clone();
    cx.node = None;

    for listener in &listeners {
        if listener.msg_id == msg.type_id() {
            (listener.listen)(listener.node.clone(), listener.slot, &msg)
        }
    }

    cx.node = Some(unsafe { mem::transmute(cx.handle.node.borrow_mut()) });
}

/// A signal to messages from an object.
pub trait Signal<M: 'static>: Sized {
    /// Emit a message from this object.
    ///
    /// This can be overriden.
    fn emit(cx: &mut Context<Self>, msg: M) {
        emit(cx, msg)
    }
}
