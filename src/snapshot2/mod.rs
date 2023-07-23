use std::{any::Any, cell::RefCell, mem, rc::Rc, sync::atomic::AtomicI32};

mod builder;
pub use builder::Builder;

mod set;
use self::{builder::MutableBuilder, set::Set};

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
    pub fn local() -> Self {
        Self::with_local(Clone::clone)
    }

    pub fn with_local<R>(f: impl FnOnce(&Self) -> R) -> R {
        LOCAL_SNAPSHOT.try_with(|local| f(&local.borrow())).unwrap()
    }

    pub fn builder(self) -> Builder {
        Builder::new(self)
    }

    pub fn builder_mut(self) -> MutableBuilder {
        MutableBuilder::new(self)
    }

    /// Take a snapshot of the current value of all states.
    pub fn take(self) -> Self {
        LOCAL_SNAPSHOT
            .try_with(|local| mem::replace(&mut *local.borrow_mut(), self))
            .unwrap()
    }
}
