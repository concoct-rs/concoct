use crate::Context;
use std::{cell::RefCell, rc::Rc};

pub fn use_ref<T: 'static>(make_value: impl FnOnce() -> T) -> Rc<T> {
    let cx = Context::current();
    let cx_ref = cx.inner.borrow();
    let scope = &mut *cx_ref.scope.as_ref().unwrap().inner.borrow_mut();

    let idx = scope.hook_idx;
    scope.hook_idx += 1;

    if let Some(any) = scope.hooks.get(idx) {
        Rc::downcast(any.clone()).unwrap()
    } else {
        let value = Rc::new(make_value());
        scope.hooks.push(value.clone());
        value
    }
}

pub fn use_state<T: 'static>(make_value: impl FnOnce() -> T) -> (Rc<T>, impl Fn(T)) {
    let cell = use_ref(|| RefCell::new(Rc::new(make_value())));

    let getter = cell.borrow().clone();

    let cx = Context::current();
    let key = cx.inner.borrow().node.unwrap();
    let setter = move |value| {
        cx.inner.borrow_mut().pending.push_back(key);
        *cell.borrow_mut() = Rc::new(value);
    };

    (getter, setter)
}
