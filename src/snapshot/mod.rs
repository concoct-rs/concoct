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

pub struct Snapshot {
    id: u64,
    invalid: HashSet<u64>,
    kind: SnapKind,
}

impl Snapshot {
    pub fn take(read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>) -> Self {
        with_current_snapshot(|snapshot| snapshot.take_nested_snapshot(read_observer))
    }

    pub fn enter<R>(self, f: impl FnOnce() -> R) -> R {
        let prev = swap_current(Some(self));
        let output = f();
        swap_current(prev);
        output
    }

    fn take_nested_snapshot(
        &mut self,
        read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
    ) -> Self {
        match self.kind {
            SnapKind::Global => {
                let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
                Self {
                    id,
                    invalid: self.invalid.clone(),
                    kind: SnapKind::ReadOnly,
                }
            }
            SnapKind::ReadOnly => todo!(),
        }
    }
}

pub enum SnapKind {
    Global,
    ReadOnly,
}

lazy_static::lazy_static! {
    static ref GLOBAL_SNAPSHOT: Mutex<Snapshot> = Mutex::new(Snapshot { id: 1, invalid: HashSet::new(), kind: SnapKind::Global });
}

static NEXT_ID: AtomicU64 = AtomicU64::new(2);

thread_local! {
    static THREAD_SNAPSHOT: RefCell<Option<Snapshot>> = RefCell::new(None);
}

pub fn swap_current(snapshot: Option<Snapshot>) -> Option<Snapshot> {
    THREAD_SNAPSHOT
        .try_with(|current| mem::replace(&mut *current.borrow_mut(), snapshot))
        .unwrap()
}

pub fn with_current_snapshot<R>(f: impl FnOnce(&mut Snapshot) -> R) -> R {
    THREAD_SNAPSHOT
        .try_with(|thread_snapshot| {
            if let Some(snapshot) = thread_snapshot.borrow_mut().as_mut() {
                f(snapshot.borrow_mut())
            } else {
                let mut snapshot = GLOBAL_SNAPSHOT.lock().unwrap();
                f(&mut *snapshot)
            }
        })
        .unwrap()
}

pub struct StateRecord<T> {
    snapshot_id: u64,
    value: T,
}

pub struct MutableState<T, U> {
    records: Vec<StateRecord<T>>,
    policy: U,
}

impl<T, U> MutableState<T, U> {
    pub fn new(value: T, policy: U) -> Self {
        Self {
            records: vec![StateRecord {
                snapshot_id: with_current_snapshot(|snapshot| snapshot.id),
                value,
            }],
            policy,
        }
    }

    /// The readable record is the valid record with the highest snapshot_id
    pub fn get(&mut self) -> &T {
        with_current_snapshot(|snapshot| readable(&self.records, snapshot.id, &snapshot.invalid))
            .unwrap()
    }

    pub fn set(&mut self, value: T)
    where
        U: MutationPolicy<T>,
    {
        let prev = current(&self.records);
        if prev.is_none() || !self.policy.is_eq(prev.unwrap(), &value) {
            with_current_snapshot(|snapshot| {
                self.records.insert(
                    0,
                    StateRecord {
                        snapshot_id: snapshot.id + 1,
                        value,
                    },
                );
            })
        }
    }
}

/// Returns the current record without notifying any read observers.
pub fn current<T>(records: &[StateRecord<T>]) -> Option<&T> {
    with_current_snapshot(|snapshot| readable(records, snapshot.id, &snapshot.invalid))
}

fn readable<'a, T>(
    records: &'a [StateRecord<T>],
    id: u64,
    invalid: &HashSet<u64>,
) -> Option<&'a T> {
    let mut iter = records.iter();
    let mut candidate: Option<&StateRecord<T>> = None;

    while let Some(record) = iter.next() {
        if is_valid(id, record.snapshot_id, invalid) {
            if let Some(candidate) = &mut candidate {
                if record.snapshot_id >= candidate.snapshot_id {
                    *candidate = record;
                }
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
    use super::{mutation_policy::ReferentialEqualityPolicy, MutableState, Snapshot};

    #[test]
    fn it_works() {
        let mut state = MutableState::new(0, ReferentialEqualityPolicy);
        state.set(1);

        let snapshot = Snapshot::take(None);
        snapshot.enter(|| {
            dbg!(state.get());
        });
    }
}
