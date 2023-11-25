use crate::{use_ref, TASK_CONTEXT};
use futures::task::LocalSpawnExt;
use futures::Future;

/// A hook that spawns a local future on current thread.
pub fn use_future<F>(f: impl FnOnce() -> F)
where
    F: Future + 'static,
{
    let _handle = use_ref(|| {
        let future = f();
        TASK_CONTEXT
            .try_with(|cx| {
                let guard = cx.borrow_mut();
                let cx = guard.as_ref().unwrap();
                let tx = cx.tx.clone();
                cx.local_pool.borrow().spawner().spawn_local(async move {
                    let _output = future.await;
                    tx.unbounded_send(Box::new(())).unwrap();
                });
            })
            .unwrap()
    });
}
