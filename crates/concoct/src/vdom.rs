use crate::{Runtime, Tree, View};
use std::{
    task::{Poll, Waker},
    time::{Duration, Instant},
};

pub fn virtual_dom(view: impl View) -> VirtualDom<impl Tree> {
    VirtualDom {
        cx: Runtime::default(),
        tree: view.into_tree(),
    }
}

pub struct VirtualDom<T> {
    cx: Runtime,
    tree: T,
}

impl<T> VirtualDom<T> {
    pub fn build(&mut self)
    where
        T: Tree,
    {
        self.cx.enter();

        unsafe { self.tree.build() }
    }

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
            let raw = inner.nodes[key];
            drop(inner);

            self.cx.enter();

            let pending = unsafe { &mut *raw };
            unsafe { pending.build() };
        }
    }
}
