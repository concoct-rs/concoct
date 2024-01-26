use crate::Scope;
use std::cell::UnsafeCell;

/// Hook to store a stateless value.
///
/// This function will only call `make_initial` once,
/// on the first render, to create the initial value.
pub fn use_ref<T, A, R: 'static>(cx: &Scope<T, A>, make_initial: impl FnOnce() -> R) -> &mut R {
    let mut node = cx.node.inner.borrow_mut();
    let idx = node.hook_idx;
    node.hook_idx += 1;

    let cell = if let Some(cell) = node.hooks.get(idx) {
        cell
    } else {
        node.hooks.push(UnsafeCell::new(Box::new(make_initial())));
        node.hooks.last().unwrap()
    };

    // Safety: All references to scope are dropped before `hook_idx` is reset.
    // This ensures unique access to each hook.
    unsafe { &mut *cell.get() }.downcast_mut().unwrap()
}
