use std::any::Any;
use crate::Context;

pub fn emit<M: 'static>(cx: &mut Context<impl Signal<M>>, msg: M) {
    for listener in &mut cx.node.listeners {
        if listener.msg_id == msg.type_id() {
            (listener.f)(&msg)
        }
    }
}

pub trait Signal<M: 'static>: Sized {
    fn emit(cx: &mut Context<Self>, msg: M) {
        emit(cx, msg)
    }
}