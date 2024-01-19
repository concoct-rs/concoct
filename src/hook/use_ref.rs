use crate::Context;
use std::rc::Rc;

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
