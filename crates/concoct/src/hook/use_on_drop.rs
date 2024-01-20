use super::use_ref;
use crate::Context;

pub fn use_on_drop(on_drop: impl FnMut() + 'static) {
    use_ref(|| {
        Context::current()
            .inner
            .borrow()
            .scope
            .as_ref()
            .unwrap()
            .inner
            .borrow_mut()
            .droppers
            .push(Box::new(on_drop))
    });
}
