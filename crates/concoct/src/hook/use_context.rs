use crate::Runtime;
use std::{any::TypeId, rc::Rc};

/// Hook to provide a context.
pub fn use_provider<T: 'static>(value: T) {
    let cx = Runtime::current();
    let cx_ref = cx.inner.borrow();
    let mut scope = cx_ref.scope.as_ref().unwrap().inner.borrow_mut();

    scope.contexts.insert(TypeId::of::<T>(), Rc::new(value));
}

/// Hook to get a context from its type.
pub fn use_context<T: 'static>() -> Option<Rc<T>> {
    Runtime::current()
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
