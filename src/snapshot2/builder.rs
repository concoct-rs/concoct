use super::{Inner, Snapshot, SnapshotKind, NEXT_SNAPSHOT_ID};
use std::{any::Any, rc::Rc, sync::atomic::Ordering};

pub struct Builder {
    parent: Snapshot,
    read_observer: Option<Rc<dyn Fn(&dyn Any)>>,
}

impl Builder {
    pub(super) fn new(parent: Snapshot) -> Self {
        Self {
            parent,
            read_observer: None,
        }
    }

    pub fn read_observer(mut self, observer: Option<impl Into<Rc<dyn Fn(&dyn Any)>>>) -> Self {
        self.read_observer = observer.map(Into::into);
        self
    }

    pub fn build(self) -> Snapshot {
        let id = NEXT_SNAPSHOT_ID.fetch_add(1, Ordering::SeqCst);
        let mut invalid = self.parent.inner.invalid.clone();
        for idx in self.parent.inner.id..id {
            invalid.set(idx);
        }

        Snapshot {
            inner: Rc::new(Inner {
                kind: SnapshotKind::Immutable,
                id,
                read_observer: self.read_observer,
                invalid,
                parent: Some(self.parent),
            }),
        }
    }
}
