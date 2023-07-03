use super::{scope::LOCAL_SCOPE, Operation, LOCAL_SNAPSHOT};
use std::{
    any::Any,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

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
