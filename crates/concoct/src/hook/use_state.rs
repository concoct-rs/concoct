use super::use_ref;
use crate::Context;
use std::{cell::RefCell, rc::Rc};

pub fn use_state<T: Clone + 'static>(make_value: impl FnOnce() -> T) -> (T, Rc<dyn Fn(T)>) {
    let cell = use_ref(|| RefCell::new(make_value()));
    let getter = cell.borrow().clone();

    let cx = Context::current();
    let key = cx.inner.borrow().node.unwrap();
    let setter = move |value| {
        *cell.borrow_mut() = value;

        let mut cx_ref = cx.inner.borrow_mut();
        cx_ref.pending.push_back(key);
        if let Some(waker) = cx_ref.waker.take() {
            waker.wake();
        }
    };

    (getter, Rc::new(setter))
}