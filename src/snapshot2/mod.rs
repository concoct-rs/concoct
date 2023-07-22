use std::{
    any::Any,
    cell::RefCell,
    mem,
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};

mod set;
use self::set::Set;

pub enum SnapshotKind {
    Immutable,
    Mutable {
        write_observer: Option<Rc<dyn Fn(&dyn Any)>>,
    },
}

thread_local! {
    static LOCAL_SNAPSHOT: RefCell<Snapshot> = RefCell::new(Snapshot {
        inner: Rc::new(Inner {
            kind: SnapshotKind::Mutable { write_observer: None },
            parent: None,
            read_observer: None,
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
    parent: Option<Snapshot>,
    read_observer: Option<Rc<dyn Fn(&dyn Any)>>,
}

#[derive(Clone)]
pub struct Snapshot {
    inner: Rc<Inner>,
}

impl Snapshot {
    /// Take a snapshot of the current value of all states.
    pub fn take(read_observer: Option<Rc<dyn Fn(&dyn Any)>>) -> Self {
        LOCAL_SNAPSHOT
            .try_with(|local| {
                let mut local = local.borrow_mut();
                let new = local.take_nested(read_observer);
                mem::replace(&mut *local, new)
            })
            .unwrap()
    }

    /// Take a nested snapshot of the current value of all states.
    pub fn take_nested(&self, read_observer: Option<Rc<dyn Fn(&dyn Any)>>) -> Self {
        let id = NEXT_SNAPSHOT_ID.fetch_add(1, Ordering::SeqCst);
        let mut invalid = self.inner.invalid.clone();
        for idx in self.inner.id..id {
            invalid.set(idx);
        }

        Self {
            inner: Rc::new(Inner {
                kind: SnapshotKind::Immutable,
                id,
                read_observer,
                invalid,
                parent: Some(self.clone()),
            }),
        }
    }

    /// Take a nested snapshot of the current value of all states.
    pub fn take_nested_mut(
        &self,
        read_observer: Option<Rc<dyn Fn(&dyn Any)>>,
        write_observer: Option<Rc<dyn Fn(&dyn Any)>>,
    ) -> Self {
        fn merge(
            prev: Option<Rc<dyn Fn(&dyn Any)>>,
            new: Option<Rc<dyn Fn(&dyn Any)>>,
        ) -> Option<Rc<dyn Fn(&dyn Any)>> {
            if let Some(previous) = prev {
                if let Some(f) = new {
                    let prev = previous.clone();
                    let f: Rc<dyn Fn(&dyn Any)> = Rc::new(move |value| {
                        prev(value);
                        f(value);
                    });
                    Some(f)
                } else {
                    Some(previous.clone())
                }
            } else {
                new
            }
        }

        let SnapshotKind::Mutable {
            write_observer: ref previous,
        } = self.inner.kind
        else {
            todo!()
        };
        let write_observer = merge(previous.clone(), write_observer);
        let read_observer = merge(self.inner.read_observer.clone(), read_observer);

        let id = NEXT_SNAPSHOT_ID.fetch_add(1, Ordering::SeqCst);
        let mut invalid = self.inner.invalid.clone();
        for idx in self.inner.id..id {
            invalid.set(idx);
        }

        Self {
            inner: Rc::new(Inner {
                kind: SnapshotKind::Mutable { write_observer },
                id,
                read_observer,
                invalid,
                parent: Some(self.clone()),
            }),
        }
    }
}
