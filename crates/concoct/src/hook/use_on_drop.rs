use super::use_ref;
use crate::Runtime;

/// Hook to store a function that's triggered on removal of the current `View`.
pub fn use_on_drop(on_drop: impl FnMut() + 'static) {
    use_ref(|| {
        Runtime::current()
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
