use std::{
    any::Any,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard, RwLock},
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

struct Operation {
    value: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
    f: Box<dyn FnMut(&mut dyn Any) + Send + Sync>,
}

static TX: RwLock<Option<UnboundedSender<Operation>>> = RwLock::new(None);

pub struct Snapshot {
    rx: UnboundedReceiver<Operation>,
}

impl Snapshot {
    pub fn take() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        *TX.write().unwrap() = Some(tx);
        Self { rx }
    }

    pub fn apply(&mut self) {
        while let Ok(mut next) = self.rx.try_recv() {
            let mut value = next.value.lock().unwrap();
            (next.f)(&mut *value)
        }
    }
}

pub struct State<T> {
    value: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
    _marker: PhantomData<T>,
}

impl<T> State<T> {
    pub fn get(&self) -> Guard<T> {
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
        TX.read()
            .unwrap()
            .as_ref()
            .unwrap()
            .send(Operation {
                value: self.value.clone(),
                f: Box::new(move |any| f(&mut *any.downcast_mut().unwrap())),
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
