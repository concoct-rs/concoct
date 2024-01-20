use super::use_ref;
use crate::Runtime;
use std::cell::RefCell;

/// Hook to create render state.
///
/// This function will only call `make_value` once, on the first render,
/// to create the initial state.
/// The returned function can be called to set the state.
///
/// ```no_run
/// use concoct::hook::use_state;
///
/// let (count, set_count) = use_state(|| 0);
/// assert_eq!(count, 0);
/// ```
pub fn use_state<T: Clone + 'static>(
    make_value: impl FnOnce() -> T,
) -> (T, impl Fn(T) + Clone + 'static) {
    let cell = use_ref(|| RefCell::new(make_value()));
    let getter = cell.borrow().clone();

    let cx = Runtime::current();
    let key = cx.inner.borrow().node.unwrap();
    let setter = move |value| {
        *cell.borrow_mut() = value;

        let mut cx_ref = cx.inner.borrow_mut();
        cx_ref.pending.push_back(key);
        if let Some(waker) = cx_ref.waker.take() {
            waker.wake();
        }
    };

    (getter, setter)
}
