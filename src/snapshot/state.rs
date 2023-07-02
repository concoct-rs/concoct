use super::{
    mutation_policy::{MutationPolicy, ReferentialEqualityPolicy},
    with_current_snapshot, INVALID_SNAPSHOT,
};
use std::collections::HashSet;

struct StateRecord<T> {
    snapshot_id: u64,
    value: T,
}

pub struct State<T, U> {
    records: Vec<StateRecord<T>>,
    policy: U,
}

impl<T> State<T, ReferentialEqualityPolicy> {
    pub fn new(value: T) -> Self {
        Self::new_with_policy(value, ReferentialEqualityPolicy)
    }
}

impl<T, U> State<T, U> {
    pub fn new_with_policy(value: T, policy: U) -> Self {
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
fn current<T>(records: &[StateRecord<T>]) -> Option<&T> {
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
