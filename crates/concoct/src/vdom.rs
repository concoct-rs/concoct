use crate::{Runtime, Tree};
use std::{
    task::{Poll, Waker},
    time::{Duration, Instant},
};

/// A virtual dom that renders a view on any backend.
pub struct VirtualDom<T> {
    cx: Runtime,
    tree: T,
}

impl<T> VirtualDom<T> {
    /// Create a new virtual dom from a tree.
    pub fn new(tree: T) -> Self {
        VirtualDom {
            cx: Runtime::default(),
            tree,
        }
    }

    /// Build the initial virtual dom.
    pub fn build(&mut self)
    where
        T: Tree,
    {
        self.cx.enter();

        // Safety: Context is dropped when the tree is
        unsafe { self.tree.build() }
    }

    /// Rebuild the virtual dom.
    pub async fn rebuild(&mut self) {
        futures::future::poll_fn(|cx| {
            self.try_rebuild_with_limit_inner(None, Some(cx.waker().clone()));

            Poll::Pending
        })
        .await
    }

    pub async fn rebuild_with_limit(&mut self, limit: Duration) {
        futures::future::poll_fn(|cx| {
            let instant = Instant::now() + limit;
            self.try_rebuild_with_limit_inner(Some(instant), Some(cx.waker().clone()));
            Poll::Pending
        })
        .await
    }

    pub fn try_rebuild(&mut self) {
        self.try_rebuild_with_limit_inner(None, None)
    }

    pub fn try_rebuild_with_limit(&mut self, limit: Duration) {
        let instant = Instant::now() + limit;
        self.try_rebuild_with_limit_inner(Some(instant), None)
    }

    fn try_rebuild_with_limit_inner(&mut self, limit: Option<Instant>, waker: Option<Waker>) {
        let mut inner = self.cx.inner.borrow_mut();
        inner.limit = limit;
        inner.waker = waker;

        if let Some(key) = inner.pending.pop_front() {
            if let Some(raw) = inner.nodes.get(key).copied() {
                drop(inner);

                self.cx.enter();

                // Safety: `raw` is guaranteed to be an `&mut dyn Tree`.
                unsafe { (&mut *raw).build() };
            }
        }
    }
}
