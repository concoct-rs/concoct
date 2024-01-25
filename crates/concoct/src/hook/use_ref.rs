use crate::Scope;
use std::cell::UnsafeCell;

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
    unsafe { &mut *cell.get() }.downcast_mut().unwrap()
}
