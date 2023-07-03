use std::{
    any::Any,
    cell::RefCell,
    iter,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

mod scope;
pub use scope::Scope;

mod state;
pub use state::{Guard, State};

mod task;
pub use task::{spawn, Task};

pub struct Snapshot {
    rx: UnboundedReceiver<Operation>,
}

impl Snapshot {
    pub fn enter() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        LOCAL_SNAPSHOT
            .try_with(|local| *local.borrow_mut() = Some(LocalSnapshot { tx }))
            .unwrap();

        Self { rx }
    }

    pub async fn apply(&mut self) -> impl Iterator<Item = u64> + '_ {
        let mut first = self.rx.recv().await.unwrap();
        first.apply();

        iter::once(first.state_id).chain(self.apply_pending())
    }

    pub fn apply_pending(&mut self) -> impl Iterator<Item = u64> + '_ {
        iter::from_fn(|| {
            self.rx.try_recv().ok().map(|mut op| {
                op.apply();
                op.state_id
            })
        })
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        LOCAL_SNAPSHOT
            .try_with(|local| *local.borrow_mut() = None)
            .unwrap();
    }
}

#[derive(Clone)]
struct LocalSnapshot {
    tx: UnboundedSender<Operation>,
}

impl LocalSnapshot {
    pub fn enter<R>(self, f: impl FnOnce() -> R) -> R {
        LOCAL_SNAPSHOT
            .try_with(|local| *local.borrow_mut() = Some(self))
            .unwrap();
        let output = f();
        LOCAL_SNAPSHOT
            .try_with(|local| *local.borrow_mut() = None)
            .unwrap();
        output
    }
}

thread_local! {
    static LOCAL_SNAPSHOT: RefCell<Option<LocalSnapshot>> = RefCell::new(None);
}

struct Operation {
    state_id: u64,
    value: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
    f: Box<dyn FnMut(&mut dyn Any) + Send + Sync>,
}

impl Operation {
    fn apply(&mut self) {
        let mut guard = self.value.lock().unwrap();
        let value: &mut dyn Any = guard.as_mut();
        (self.f)(&mut *value)
    }
}

#[cfg(test)]
mod tests {
    use super::{Scope, Snapshot};
    use crate::snapshot::state::State;

    #[test]
    fn it_works() {
        let mut snapshot = Snapshot::enter();

        Scope::default().enter(|| {
            let state = State::new(0);

            state.update(|x| *x = 1);
            assert_eq!(*state.get(), 0);

            for _id in snapshot.apply_pending() {}
            assert_eq!(*state.get(), 1);
        });
    }
}
