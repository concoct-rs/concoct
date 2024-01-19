use crate::Context;
use std::{any::TypeId, rc::Rc};

pub fn use_provider<T: 'static>(value: T) {
    let cx = Context::current();
    let cx_ref = cx.inner.borrow();
    let mut scope = cx_ref.scope.as_ref().unwrap().inner.borrow_mut();

    scope.contexts.insert(TypeId::of::<T>(), Rc::new(value));
}

pub fn use_context<T: 'static>() -> Option<Rc<T>> {
    Context::current()
        .inner
        .borrow()
        .scope
        .as_ref()
        .unwrap()
        .inner
        .borrow()
        .contexts
        .get(&TypeId::of::<T>())
        .map(|rc| Rc::downcast(rc.clone()).unwrap())
}
