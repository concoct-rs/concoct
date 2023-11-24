use crate::LocalContext;
use std::{any::Any, cell::RefCell, rc::Rc};

pub fn use_hook<T: 'static>(make_value: impl FnOnce() -> T) -> Rc<RefCell<dyn Any>> {
    let cx = LocalContext::current();
    let mut inner = cx.inner.borrow_mut();
    let mut hooks = inner.hooks.borrow_mut();

    let value = if let Some(hook) = hooks.get(inner.idx) {
        let value = hook.clone();

        value
    } else {
        hooks.push(Rc::new(RefCell::new(make_value())));
        hooks.last().as_deref().unwrap().clone()
    };

    drop(hooks);
    inner.idx += 1;

    value
}
