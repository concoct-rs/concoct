use std::{
    any::Any,
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashSet,
    mem,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

use self::mutation_policy::MutationPolicy;

pub mod mutation_policy;

pub trait Snapshot {
    fn id(&self) -> u64;

    fn invalid(&self) -> &HashSet<u64>;

    fn take_nested_snapshot(
        &mut self,
        read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
    ) -> Box<dyn Snapshot>;
}

lazy_static::lazy_static! {
    static ref GLOBAL_SNAPSHOT: Mutex<GlobalSnapshot> = Mutex::new(GlobalSnapshot { id: 1, invalid: HashSet::new() });
}

static NEXT_ID: AtomicU64 = AtomicU64::new(2);

thread_local! {
    static THREAD_SNAPSHOT: RefCell<Option<Box<dyn Snapshot>>> = RefCell::new(None);
}

pub fn snapshot(read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>) -> Box<dyn Snapshot> {
    with_current_snapshot(|snapshot| snapshot.take_nested_snapshot(read_observer))
}

pub fn swap_current(snapshot: Option<Box<dyn Snapshot>>) -> Option<Box<dyn Snapshot>> {
    THREAD_SNAPSHOT
        .try_with(|current| mem::replace(&mut *current.borrow_mut(), snapshot))
        .unwrap()
}

pub fn enter<R>(snapshot: Box<dyn Snapshot>, f: impl FnOnce() -> R) -> R {
    let prev = swap_current(Some(snapshot));
    let output = f();
    swap_current(prev);
    output
}

pub struct GlobalSnapshot {
    id: u64,
    invalid: HashSet<u64>,
}

impl Snapshot for GlobalSnapshot {
    fn take_nested_snapshot(
        &mut self,
        read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
    ) -> Box<dyn Snapshot> {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Box::new(ReadOnlySnapshot { id, read_observer })
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn invalid(&self) -> &HashSet<u64> {
        &self.invalid
    }
}

pub fn with_current_snapshot<R>(f: impl FnOnce(&mut dyn Snapshot) -> R) -> R {
    THREAD_SNAPSHOT
        .try_with(|thread_snapshot| {
            if let Some(snapshot) = thread_snapshot.borrow_mut().as_deref_mut() {
                f(snapshot.borrow_mut())
            } else {
                let mut snapshot = GLOBAL_SNAPSHOT.lock().unwrap();
                f(&mut *snapshot)
            }
        })
        .unwrap()
}

pub struct ReadOnlySnapshot {
    id: u64,
    // invalid = invalid,
    read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
}

impl Snapshot for ReadOnlySnapshot {
    fn take_nested_snapshot(
        &mut self,
        read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
    ) -> Box<dyn Snapshot> {
        todo!()
    }

    fn id(&self) -> u64 {
        todo!()
    }

    fn invalid(&self) -> &HashSet<u64> {
        todo!()
    }
}

pub struct StateRecord<T> {
    snapshot_id: u64,
    value: T,
}

pub struct SnapshotMutableState<T, U> {
    records: Vec<StateRecord<T>>,
    policy: U,
}

impl<T, U> SnapshotMutableState<T, U> {
    pub fn new(value: T, policy: U) -> Self {
        Self {
            records: vec![StateRecord {
                snapshot_id: with_current_snapshot(|snapshot| snapshot.id()),
                value,
            }],
            policy,
        }
    }

    /// The readable record is the valid record with the highest snapshot_id
    pub fn get(&mut self) -> &T {
        with_current_snapshot(|snapshot| readable(&self.records, snapshot.id(), snapshot.invalid()))
            .unwrap()
    }

    pub fn set(&mut self, value: T)
    where
        U: MutationPolicy<T>,
    {
        let prev = current(&self.records);
        if prev.is_none() || !self.policy.is_eq(prev.unwrap(), &value) {
            // TODO
        }
    }
}

/// Returns the current record without notifying any read observers.
pub fn current<T>(records: &[StateRecord<T>]) -> Option<&T> {
    with_current_snapshot(|snapshot| readable(records, snapshot.id(), snapshot.invalid()))
}

fn readable<'a, T>(
    records: &'a [StateRecord<T>],
    id: u64,
    invalid: &HashSet<u64>,
) -> Option<&'a T> {
    let mut iter = records.iter();
    let mut candidate = None;

    while let Some(record) = iter.next() {
        if is_valid(record.snapshot_id, id, invalid) {
            if let Some(candidate) = &mut candidate {
                *candidate = record;
            } else {
                candidate = Some(record);
            }
        }
    }

    candidate.map(|record| &record.value)
}

const INVALID_SNAPSHOT: u64 = 0;

/**
 * A candidate snapshot is valid if the it is less than or equal to the current snapshot
 * and it wasn't specifically marked as invalid when the snapshot started.
 *
 * All snapshot active at when the snapshot was taken considered invalid for the snapshot
 * (they have not been applied and therefore are considered invalid).
 *
 * All snapshots taken after the current snapshot are considered invalid since they where taken
 * after the current snapshot was taken.
 *
 * INVALID_SNAPSHOT is reserved as an invalid snapshot id.
 */
fn is_valid(current_snapshot: u64, candidate_snapshot: u64, invalid: &HashSet<u64>) -> bool {
    candidate_snapshot != INVALID_SNAPSHOT
        && candidate_snapshot <= current_snapshot
        && !invalid.contains(&candidate_snapshot)
}

#[cfg(test)]
mod tests {
    use super::{mutation_policy::ReferentialEqualityPolicy, SnapshotMutableState};

    #[test]
    fn it_works() {
        let mut state = SnapshotMutableState::new(0, ReferentialEqualityPolicy);
        assert_eq!(*state.get(), 0);
        state.set(1);
        assert_eq!(*state.get(), 0);
    }
}
