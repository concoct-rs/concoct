use crate::use_hook;

struct Dropper<F: FnMut()> {
    f: F,
}

impl<F: FnMut()> Drop for Dropper<F> {
    fn drop(&mut self) {
        (self.f)()
    }
}

pub fn use_on_drop(f: impl FnMut() + 'static) {
    use_hook(|| Dropper { f });
}
