use pin_project_lite::pin_project;
use std::{
    any::Any,
    cell::RefCell,
    collections::HashSet,
    future::Future,
    marker::PhantomData,
    mem,
    ops::Deref,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll},
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Default)]
pub struct Scope {
    reads: HashSet<u64>,
    writes: HashSet<u64>,
}

impl Scope {
    pub fn enter(self, f: impl FnOnce()) -> Self {
        let parent = LOCAL_SCOPE
            .try_with(|scope| mem::replace(&mut *scope.borrow_mut(), Some(self)))
            .unwrap();

        f();

        LOCAL_SCOPE
            .try_with(|scope| mem::replace(&mut *scope.borrow_mut(), parent))
            .unwrap()
            .unwrap()
    }
}

thread_local! {
    static LOCAL_SCOPE: RefCell<Option<Scope>> = RefCell::new(None);
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

    pub fn apply(&mut self) {
        while let Ok(mut op) = self.rx.try_recv() {
            op.apply();
        }
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        LOCAL_SNAPSHOT
            .try_with(|local| *local.borrow_mut() = None)
            .unwrap();
    }
}

struct Operation {
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

pub struct State<T> {
    id: u64,
    value: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
    _marker: PhantomData<T>,
}

impl<T> State<T> {
    pub fn new(value: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            id: 0,
            value: Arc::new(Mutex::new(Box::new(value))),
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> Guard<T> {
        LOCAL_SCOPE
            .try_with(|scope| {
                scope.borrow_mut().as_mut().unwrap().reads.insert(self.id);
            })
            .unwrap();

        let mutex = self.value.lock().unwrap();
        Guard {
            mutex,
            _marker: PhantomData,
        }
    }

    pub fn update(&self, mut f: impl FnMut(&mut T) + Send + Sync + 'static)
    where
        T: Send + Sync + 'static,
    {
        LOCAL_SCOPE
            .try_with(|scope| {
                scope.borrow_mut().as_mut().unwrap().writes.insert(self.id);
            })
            .unwrap();

        LOCAL_SNAPSHOT
            .try_with(|local| {
                local
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .tx
                    .send(Operation {
                        value: self.value.clone(),
                        f: Box::new(move |any| f(any.downcast_mut().unwrap())),
                    })
                    .unwrap()
            })
            .unwrap();
    }
}

pub struct Guard<'a, T> {
    mutex: MutexGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<T>,
}

impl<'a, T: 'static> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.mutex.downcast_ref().unwrap()
    }
}

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

#[cfg(test)]
mod tests {
    use super::{Scope, Snapshot};
    use crate::State;

    #[test]
    fn it_works() {
        let mut snapshot = Snapshot::enter();

        Scope::default().enter(|| {
            let state = State::new(0);

            state.update(|x| *x = 1);
            assert_eq!(*state.get(), 0);

            snapshot.apply();
            assert_eq!(*state.get(), 1);
        });
    }
}
