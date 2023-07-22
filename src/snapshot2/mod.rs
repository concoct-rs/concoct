use std::{
    cell::RefCell,
    mem,
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};

mod set;
use self::set::Set;

pub enum SnapshotKind {
    Immutable,
    NestedImmutable { parent: Snapshot },
}

thread_local! {
    static LOCAL_SNAPSHOT: RefCell<Snapshot> = RefCell::new(Snapshot {
        inner: Rc::new(Inner {
            kind: SnapshotKind::Immutable,
            id: 0,
            invalid: Set::empty(),
        }),
    });
}

static NEXT_SNAPSHOT_ID: AtomicI32 = AtomicI32::new(0);

struct Inner {
    kind: SnapshotKind,
    id: i32,
    invalid: Set,
}

#[derive(Clone)]
pub struct Snapshot {
    inner: Rc<Inner>,
}

impl Snapshot {
    /// Take a snapshot of the current value of all states.
    pub fn take() -> Self {
        LOCAL_SNAPSHOT
            .try_with(|local| {
                let mut local = local.borrow_mut();
                let new = local.take_nested();
                mem::replace(&mut *local, new)
            })
            .unwrap()
    }

    /// Take a nested snapshot of the current value of all states.
    pub fn take_nested(&self) -> Self {
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
