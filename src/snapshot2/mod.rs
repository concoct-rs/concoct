use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};

#[derive(Clone)]
pub struct SnapshotIdSet {
    // Bit set from (lowerBound + 64)-(lowerBound+127) of the set
    upper_set: i64,

    // Bit set from (lowerBound)-(lowerBound+63) of the set
    lower_set: i64,

    // Lower bound of the bit set. All values above lowerBound+127 are clear.
    // Values between lowerBound and lowerBound+127 are recorded in lowerSet and upperSet
    lower_bound: i32,

    // A sorted array of the index of bits set below lowerBound
    below_bound: Option<Vec<i32>>,
}

impl SnapshotIdSet {
    pub const fn empty() -> Self {
        Self {
            upper_set: 0,
            lower_set: 0,
            lower_bound: 0,
            below_bound: None,
        }
    }

    /// Check if the bit at `index` is set.
    pub fn is_set(&self, index: i32) -> bool {
        let offset = index - self.lower_bound;
        if offset >= 0 && offset < i64::BITS as _ {
            (1 << offset) & self.lower_set != 0
        } else if offset >= i64::BITS as _ && offset < (i64::BITS as i32) * 2 {
            (1 << (offset - i64::BITS as i32)) & self.upper_set != 0
        } else if offset > 0 {
            false
        } else if let Some(ref below_bound) = self.below_bound {
            below_bound.binary_search(&index).is_ok()
        } else {
            false
        }
    }

    pub fn set(&mut self, index: i32) {}
}

pub enum SnapshotKind {
    Immutable,
    NestedImmutable { parent: Snapshot },
}

struct Inner {
    kind: SnapshotKind,
    id: i32,
    invalid: SnapshotIdSet,
}

thread_local! {
    static LOCAL_SNAPSHOT: RefCell<Snapshot> = RefCell::new(Snapshot {
        inner: Rc::new(Inner {
            kind: SnapshotKind::Immutable,
            id: 0,
            invalid: SnapshotIdSet::empty(),
        }),
    });
}

static NEXT_SNAPSHOT_ID: AtomicI32 = AtomicI32::new(0);

#[derive(Clone)]
pub struct Snapshot {
    inner: Rc<Inner>,
}

impl Snapshot {
    pub fn take_snapshot() -> Self {
        LOCAL_SNAPSHOT
            .try_with(|local| local.borrow().take_nested_snapshot())
            .unwrap()
    }

    pub fn take_nested_snapshot(&self) -> Self {
        let id = NEXT_SNAPSHOT_ID.fetch_add(1, Ordering::SeqCst);
        let mut invalid = self.inner.invalid.clone();
        for idx in self.inner.id..id {
            invalid.set(idx);
        }

        Self {
            inner: Rc::new(Inner {
                kind: SnapshotKind::NestedImmutable {
                    parent: self.clone(),
                },
                id,
                invalid,
            }),
        }
    }
}
