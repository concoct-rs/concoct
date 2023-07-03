use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::snapshot::LocalSnapshot;

pin_project! {
    pub struct Task<F> {
        local_snapshot: LocalSnapshot,
        #[pin]
        future: F
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
