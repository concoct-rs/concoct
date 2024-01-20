use crate::{Tree, View};
use rustc_hash::FxHasher;
use std::{
    any::Any,
    hash::{Hash, Hasher},
};

pub fn memo<B>(input: impl Hash, body: B) -> Memo<B> {
    let mut hasher = FxHasher::default();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    Memo { hash, body }
}

pub struct Memo<B> {
    hash: u64,
    body: B,
}

impl<B: View> View for Memo<B> {
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

    unsafe fn rebuild(&mut self, last: &mut dyn Any) {
        let last = last.downcast_mut::<Self>().unwrap();
        if self.hash != last.hash {
            self.body.rebuild(&mut last.body)
        }
    }

    unsafe fn remove(&mut self) {
        self.body.remove()
    }
}
