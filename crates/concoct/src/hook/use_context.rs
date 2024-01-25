use crate::Scope;
use std::{any::TypeId, rc::Rc};

pub fn use_context<T, A, R: 'static>(cx: &Scope<T, A>) -> Rc<R> {
    let contexts = cx.contexts.borrow();
    let rc = contexts.get(&TypeId::of::<R>()).unwrap();
    Rc::downcast(rc.clone()).unwrap()
}
