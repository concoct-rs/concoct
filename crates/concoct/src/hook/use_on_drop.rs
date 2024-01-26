use super::use_ref;
use crate::Scope;

struct Dropper<F: FnOnce()> {
    f: Option<F>,
}

impl<F: FnOnce()> Drop for Dropper<F> {
    fn drop(&mut self) {
        self.f.take().unwrap()()
    }
}

/// Hook to call a funtion when this scope has dropped.
pub fn use_on_drop<T, A>(cx: &Scope<T, A>, on_drop: impl FnOnce() + 'static) {
    use_ref(cx, || Dropper { f: Some(on_drop) });
}
