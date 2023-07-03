use dashmap::{mapref::one::Ref, DashMap, DashSet};
use lazy_static::lazy_static;
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Deref,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_SNAPSHOT_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_STATE_ID: AtomicU64 = AtomicU64::new(0);

struct GlobalSnapshot {
    id: AtomicU64,
    states: DashMap<u64, Shared>,
    modified: DashSet<u64>,
}

lazy_static! {
    static ref GLOBAL: GlobalSnapshot = GlobalSnapshot {
        id: AtomicU64::new(NEXT_SNAPSHOT_ID.fetch_add(1, Ordering::SeqCst)),
        states: DashMap::new(),
        modified: DashSet::new()
    };
}

pub fn advance() {
    let id = NEXT_STATE_ID.fetch_add(1, Ordering::SeqCst);
    GLOBAL.id.store(id, Ordering::SeqCst);
}

thread_local! {
    static LOCAL_SNAPSHOTS: Vec<Snapshot> = Vec::new();
}

pub struct Snapshot {
    states: HashMap<u64, Shared>,
    modified: HashSet<u64>,
}

impl Snapshot {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            modified: HashSet::new(),
        }
    }

    pub fn apply(self) {
        for (state_id, shared) in self.states.into_iter() {
            GLOBAL.states.insert(state_id, shared);
        }

        for state_id in self.modified {
            GLOBAL.modified.insert(state_id);
        }

        advance();
    }
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
        GLOBAL.states.insert(id, shared);

        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> StateRef<T> {
        StateRef {
            shared: GLOBAL.states.get(&self.id).unwrap(),
            _marker: PhantomData,
        }
    }

    pub fn set(&mut self, value: T)
    where
        T: Send + Sync + 'static,
    {
        let mut shared = GLOBAL.states.get_mut(&self.id).unwrap();
        shared.records.push(Record {
            snapshot_id: GLOBAL.id.load(Ordering::SeqCst) + 1,
            value: Box::new(value),
        })
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
    use crate::{
        snapshot::{advance, Snapshot, GLOBAL},
        State,
    };

    #[test]
    fn it_works() {
        let mut state = State::new(0);
        assert_eq!(*state.get(), 0);

        state.set(1);
        assert_eq!(*state.get(), 0);

        advance();
        assert_eq!(*state.get(), 1);
    }
}
