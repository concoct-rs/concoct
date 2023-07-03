use super::{scope::LOCAL_SCOPE, Operation, LOCAL_SNAPSHOT};
use std::{
    any::Any,
    marker::PhantomData,
    ops::Deref,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct State<T> {
    id: u64,
    value: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
    _marker: PhantomData<T>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            value: self.value.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<T> State<T> {
    pub fn new(value: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
            value: Arc::new(Mutex::new(Box::new(value))),
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> Guard<T> {
        LOCAL_SCOPE
            .try_with(|scope| {
                scope
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .state_ids
                    .insert(self.id);
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
            .try_with(|local_scope| {
                if let Some(scope) = local_scope.borrow_mut().as_mut() {
                    scope.state_ids.insert(self.id);
                }
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
                        state_id: self.id,
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
