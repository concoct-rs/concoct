use crate::{use_hook, TASK_CONTEXT};
use futures::Future;

pub fn use_future<F: Future + 'static>(f: impl FnOnce() -> F) {
    use_hook(|| {
        let future = f();
        TASK_CONTEXT.try_with(|cx| {
            let guard = cx.borrow_mut();
            let cx = guard.as_ref().unwrap();
            let tx = cx.tx.clone();
            cx.local_set.borrow_mut().spawn_local(async move {
                future.await;
                tx.send(Box::new(())).unwrap();
            });
        })
    });
}
