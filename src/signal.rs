//! Signals

use crate::Context;
use core::any::Any;

/// The default implementation of `Signal::emit`.
pub fn emit<M: 'static>(cx: &mut Context<impl Signal<M>>, msg: M) {
    for listener in &mut cx.node.listeners {
        if listener.msg_id == msg.type_id() {
            (listener.f)(&msg)
        }
    }
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
