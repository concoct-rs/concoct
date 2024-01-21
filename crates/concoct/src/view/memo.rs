use crate::{Tree, View};
use rustc_hash::FxHasher;
use std::{
    any::Any,
    hash::{Hash, Hasher},
};

/// Memoize a view, only rendering it when some input has changed.
pub fn memo<V: View>(input: impl Hash, view: V) -> Memo<V> {
    let mut hasher = FxHasher::default();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    Memo { hash, body: view }
}

/// View for the [`memo`] function.
pub struct Memo<V> {
    hash: u64,
    body: V,
}

impl<V: View> View for Memo<V> {
    fn into_tree(self) -> impl Tree {
        Memo {
            hash: self.hash,
            body: self.body.into_tree(),
        }
    }
}

impl<T: Tree> Tree for Memo<T> {
    unsafe fn build(&mut self) {
        self.body.build()
    }

    unsafe fn rebuild(&mut self, last: &mut dyn Any, is_changed: bool) {
        let last = last.downcast_mut::<Self>().unwrap();
        self.body
            .rebuild(&mut last.body, is_changed && self.hash != last.hash)
    }

    unsafe fn remove(&mut self) {
        self.body.remove()
    }
}
