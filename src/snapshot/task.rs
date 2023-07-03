use crate::snapshot::LocalSnapshot;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::LOCAL_SNAPSHOT;

pin_project! {
    pub struct Task<F> {
        local_snapshot: LocalSnapshot,
        #[pin]
        future: F
    }
}

impl<F> Task<F> {
    pub fn new(future: F) -> Self {
        let local_snapshot = LOCAL_SNAPSHOT
            .try_with(|local_snapshot| local_snapshot.borrow().as_ref().unwrap().clone())
            .unwrap();
        Self {
            local_snapshot,
            future,
        }
    }
}

impl<F> Future for Task<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let me = self.project();
        me.local_snapshot.clone().enter(|| me.future.poll(cx))
    }
}

pub fn spawn<F>(future: F)
where
    F: Future + Send + Sync + 'static,
    F::Output: Send,
{
    tokio::spawn(Task::new(future));
}
