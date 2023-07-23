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

pub struct MutableBuilder {
    parent: Snapshot,
    read_observer: Option<Rc<dyn Fn(&dyn Any)>>,
    write_observer: Option<Rc<dyn Fn(&dyn Any)>>,
}

impl MutableBuilder {
    pub(super) fn new(parent: Snapshot) -> Self {
        Self {
            parent,
            read_observer: None,
            write_observer: None,
        }
    }

    pub fn read_observer(mut self, observer: Option<impl Into<Rc<dyn Fn(&dyn Any)>>>) -> Self {
        self.read_observer = observer.map(Into::into);
        self
    }

    pub fn write_observer(mut self, observer: Option<impl Into<Rc<dyn Fn(&dyn Any)>>>) -> Self {
        self.write_observer = observer.map(Into::into);
        self
    }

    pub fn build(self) -> Snapshot {
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
        } = self.parent.inner.kind
        else {
            todo!()
        };
        let write_observer = merge(previous.clone(), self.write_observer);
        let read_observer = merge(self.parent.inner.read_observer.clone(), self.read_observer);

        let id = NEXT_SNAPSHOT_ID.fetch_add(1, Ordering::SeqCst);
        let mut invalid = self.parent.inner.invalid.clone();
        for idx in self.parent.inner.id..id {
            invalid.set(idx);
        }

        Snapshot {
            inner: Rc::new(Inner {
                kind: SnapshotKind::Mutable { write_observer },
                id,
                read_observer,
                invalid,
                parent: Some(self.parent),
            }),
        }
    }
}
