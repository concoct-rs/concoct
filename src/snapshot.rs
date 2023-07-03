use dashmap::{mapref::one::Ref, DashMap};
use lazy_static::lazy_static;
use std::{
    any::Any,
    marker::PhantomData,
    ops::Deref,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_SNAPSHOT_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_STATE_ID: AtomicU64 = AtomicU64::new(0);

struct GlobalSnapshot {
    id: AtomicU64,
    modified: DashMap<u64, Shared>,
}

impl GlobalSnapshot {
    pub fn advance(&self) {
        let id = NEXT_STATE_ID.fetch_add(1, Ordering::SeqCst);
        self.id.store(id, Ordering::SeqCst);
    }
}

lazy_static! {
    static ref GLOBAL: GlobalSnapshot = GlobalSnapshot {
        id: AtomicU64::new(NEXT_SNAPSHOT_ID.fetch_add(1, Ordering::SeqCst)),
        modified: DashMap::new()
    };
}

struct Record {
    snapshot_id: u64,
    value: Box<dyn Any + Send + Sync>,
}

struct Shared {
    records: Vec<Record>,
}

pub struct State<T> {
    id: u64,
    _marker: PhantomData<T>,
}

impl<T> State<T> {
    pub fn new(value: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        let id = NEXT_STATE_ID.fetch_add(1, Ordering::SeqCst);
        let shared = Shared {
            records: vec![Record {
                snapshot_id: GLOBAL.id.load(Ordering::SeqCst),
                value: Box::new(value),
            }],
        };
        GLOBAL.modified.insert(id, shared);

        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> StateRef<T> {
        StateRef {
            shared: GLOBAL.modified.get(&self.id).unwrap(),
            _marker: PhantomData,
        }
    }
}

pub struct StateRef<'a, T> {
    shared: Ref<'a, u64, Shared>,
    _marker: PhantomData<T>,
}

impl<'a, T> Deref for StateRef<'a, T>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let snapshot_id = GLOBAL.id.load(Ordering::SeqCst);
        let record = self
            .shared
            .records
            .iter()
            .filter(|record| record.snapshot_id <= snapshot_id)
            .max_by_key(|record| record.snapshot_id)
            .unwrap();
        record.value.downcast_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::State;

    #[test]
    fn it_works() {
        let state = State::new(0);
        assert_eq!(*state.get(), 0);
    }
}
