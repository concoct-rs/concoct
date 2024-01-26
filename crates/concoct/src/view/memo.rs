use crate::{build_inner, hook::use_ref, rebuild_inner, Scope, View};
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

/// Memoize a view, only running it when dependencies have change.
pub fn memo<V>(dependencies: impl Hash, view: V) -> Memo<V> {
    let mut hasher = FxHasher::default();
    dependencies.hash(&mut hasher);
    let hash = hasher.finish();

    Memo { hash, view }
}

/// View for the [`memo`] function.
pub struct Memo<V> {
    hash: u64,
    view: V,
}

impl<T, A, V> View<T, A> for Memo<V>
where
    V: View<T, A>,
{
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        let mut is_init = false;
        let last_hash = use_ref(cx, || {
            is_init = true;
            self.hash
        });

        if is_init || self.hash != *last_hash {
            *last_hash = self.hash;

            if cx.node.inner.borrow().children.is_empty() {
                build_inner(&mut self.view, &cx);
            } else {
                rebuild_inner(&mut self.view, &cx);
            }
        }
    }
}
