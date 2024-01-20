use crate::Context;
use std::rc::Rc;

/// Hook to store a stateless value.
///
/// This function will only call `make_value` once, on the first render,
/// to create the initial value.
pub fn use_ref<T: 'static>(make_value: impl FnOnce() -> T) -> Rc<T> {
    let cx = Context::current();
    let cx_ref = cx.inner.borrow();
    let mut scope = cx_ref.scope.as_ref().unwrap().inner.borrow_mut();

    let idx = scope.hook_idx;
    scope.hook_idx += 1;

    if let Some(any) = scope.hooks.get(idx) {
        Rc::downcast(any.clone()).unwrap()
    } else {
        drop(scope);
        drop(cx_ref);
        let value = Rc::new(make_value());

        let cx = Context::current();
        let cx_ref = cx.inner.borrow();
        let scope = &mut *cx_ref.scope.as_ref().unwrap().inner.borrow_mut();
        scope.hooks.push(value.clone());
        value
    }
}
