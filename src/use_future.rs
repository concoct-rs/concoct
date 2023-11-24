use crate::{use_ref, UseRef, TASK_CONTEXT};
use futures::Future;
use tokio::task::{self, JoinHandle};

/// A hook that spawns a local future on current thread.
pub fn use_future<F>(f: impl FnOnce() -> F) -> UseFuture<F::Output>
where
    F: Future + 'static,
{
    let handle = use_ref(|| {
        let future = f();
        TASK_CONTEXT
            .try_with(|cx| {
                let guard = cx.borrow_mut();
                let cx = guard.as_ref().unwrap();
                let tx = cx.tx.clone();
                task::spawn_local(async move {
                    let output = future.await;
                    tx.send(Box::new(())).unwrap();
                    output
                })
            })
            .unwrap()
    });
    UseFuture { handle }
}

pub struct UseFuture<T> {
    handle: UseRef<JoinHandle<T>>,
}

impl<T: 'static> UseFuture<T> {
    pub fn abort(&self) {
        self.handle.get().abort()
    }
}

impl<T> Clone for UseFuture<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UseFuture<T> {}
