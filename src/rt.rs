use crate::{object::AnyObject, Handle, Object};
use futures::{channel::mpsc, StreamExt};
use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    rc::Rc,
};

pub(crate) struct RuntimeMessage(pub(crate) RuntimeMessageKind);

pub(crate) enum RuntimeMessageKind {
    Emit {
        key: DefaultKey,
        msg: Box<dyn Any>,
        f: Box<dyn FnOnce(&mut dyn AnyObject, DefaultKey, &dyn Any)>,
    },
    Handle {
        key: DefaultKey,
        f: Box<dyn FnOnce(&mut dyn AnyObject)>,
    },
    Listen {
        key: DefaultKey,
        type_id: TypeId,
        f: Rc<RefCell<dyn FnMut(&dyn Any)>>,
        listen_f: Box<dyn FnOnce(&mut dyn AnyObject)>,
    },
    Remove {
        key: DefaultKey,
    },
}

pub(crate) struct Inner {
    pub(crate) objects: SlotMap<DefaultKey, Rc<RefCell<dyn AnyObject>>>,
    pub(crate) listeners: FxHashMap<(DefaultKey, TypeId), Vec<Rc<RefCell<dyn FnMut(&dyn Any)>>>>,
    pub(crate) rx: mpsc::UnboundedReceiver<RuntimeMessage>,
}

/// Local reactive object runtime.
///
/// This executes tasks in a thread-per-core fashion.
#[derive(Clone)]
pub struct Runtime {
    pub(crate) inner: Rc<RefCell<Inner>>,
    pub(crate) tx: mpsc::UnboundedSender<RuntimeMessage>,
}

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::default();
}

impl Default for Runtime {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded();
        Self {
            inner: Rc::new(RefCell::new(Inner {
                objects: SlotMap::new(),
                listeners: FxHashMap::default(),
                rx,
            })),
            tx,
        }
    }
}

impl Runtime {
    pub fn current() -> Self {
        Self::try_current().unwrap()
    }

    pub fn try_current() -> Option<Self> {
        CURRENT
            .try_with(|cell| cell.borrow().clone())
            .ok()
            .flatten()
    }

    pub fn enter(&self) -> RuntimeGuard {
        CURRENT
            .try_with(|cell| *cell.borrow_mut() = Some(self.clone()))
            .unwrap();

        RuntimeGuard { _priv: () }
    }

    pub fn start<T>(&self, object: T) -> Handle<T>
    where
        T: Object + 'static,
    {
        let key = self
            .inner
            .borrow_mut()
            .objects
            .insert(Rc::new(RefCell::new(object)));

        let object = self.inner.borrow().objects[key].clone();
        let handle = Handle::new(key, self.tx.clone());
        object.borrow_mut().started_any(handle.guard.clone());
        handle
    }

    pub async fn run(&self) {
        let mut me = self.inner.borrow_mut();
        if let Some(msg) = me.rx.next().await {
            drop(me);
            self.run_inner(msg);

            self.try_run();
        }
    }

    pub fn try_run(&self) {
        loop {
            let mut me = self.inner.borrow_mut();
            if let Ok(Some(msg)) = me.rx.try_next() {
                drop(me);
                self.run_inner(msg);
            } else {
                break;
            }
        }
    }

    fn run_inner(&self, msg: RuntimeMessage) {
        match msg.0 {
            RuntimeMessageKind::Emit { key, msg, f } => {
                let me = self.inner.borrow();
                let object = me.objects[key].clone();
                drop(me);

                let mut object_ref = object.borrow_mut();
                f(&mut *object_ref, key, &*msg);
                drop(object_ref);

                let me = self.inner.borrow();
                let listeners = me.listeners.get(&(key, (&*msg).type_id())).cloned();
                drop(me);

                if let Some(listeners) = listeners {
                    for listener in listeners {
                        listener.borrow_mut()(&*msg)
                    }
                }
            }
            RuntimeMessageKind::Handle { key, f } => {
                let me = self.inner.borrow();
                let object = me.objects[key].clone();
                drop(me);

                let mut object_ref = object.borrow_mut();
                f(&mut *object_ref);
            }
            RuntimeMessageKind::Listen {
                key,
                type_id,
                f,
                listen_f,
            } => {
                self.inner
                    .borrow_mut()
                    .listeners
                    .insert((key, type_id), vec![f]);

                let object = self.inner.borrow().objects[key].clone();
                listen_f(&mut *object.borrow_mut());
            }
            RuntimeMessageKind::Remove { key } => {
                self.inner.borrow_mut().objects.remove(key);
            }
        }
    }
}

pub struct RuntimeGuard {
    _priv: (),
}

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        CURRENT.try_with(|cell| cell.borrow_mut().take()).unwrap();
    }
}
