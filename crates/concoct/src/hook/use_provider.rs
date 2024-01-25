use crate::Scope;
use std::{any::TypeId, rc::Rc};

use super::use_ref;

pub fn use_provider<T, A, R: 'static>(cx: &Scope<T, A>, make_initial: impl FnOnce() -> R) {
    use_ref(cx, || {
        cx.contexts
            .borrow_mut()
            .insert(TypeId::of::<R>(), Rc::new(make_initial()))
    });
}
