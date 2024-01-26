use super::use_ref;
use crate::Scope;
use std::{any::TypeId, rc::Rc};

/// Hook to provide a context.
pub fn use_provider<T, A, R: 'static>(cx: &Scope<T, A>, make_initial: impl FnOnce() -> R) {
    let value = use_ref(cx, || Rc::new(make_initial()));

    cx.contexts
        .borrow_mut()
        .insert(TypeId::of::<R>(), value.clone());
}
