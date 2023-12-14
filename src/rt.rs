use crate::{
    handle::{BindHandle, HandleGuard, ListenerGuard},
    object::AnyObject,
    Handle, Object,
};
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
    Remove {
        key: DefaultKey,
    },
    Listen {
        id: u64,
        key: DefaultKey,
        type_id: TypeId,
        f: Rc<RefCell<dyn FnMut(&dyn Any)>>,
        listen_f: Box<dyn FnOnce(&mut dyn AnyObject)>,
        listener: Option<HandleGuard>,
    },
    RemoveListener {
        id: u64,
        key: DefaultKey,
        type_id: TypeId,
    },
}

pub(crate) struct Node {
    pub(crate) object: Box<dyn AnyObject>,
    pub(crate) listener_guards: Vec<ListenerGuard>,
}

pub(crate) struct Inner {
    pub(crate) nodes: SlotMap<DefaultKey, Rc<RefCell<Node>>>,
    pub(crate) listeners:
        FxHashMap<(DefaultKey, TypeId), Vec<(u64, Rc<RefCell<dyn FnMut(&dyn Any)>>)>>,
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
                nodes: SlotMap::new(),
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
            .nodes
            .insert(Rc::new(RefCell::new(Node {
                object: Box::new(object),
                listener_guards: Vec::new(),
            })));

        let node = self.inner.borrow().nodes[key].clone();
        let handle = Handle::new(key, self.tx.clone());
        node.borrow_mut().object.started_any(handle.guard.clone());
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
                let node = me.nodes[key].clone();
                drop(me);

                f(&mut *node.borrow_mut().object, key, &*msg);
                drop(node);

                let me = self.inner.borrow();
                let listeners = me.listeners.get(&(key, (&*msg).type_id())).cloned();
                drop(me);

                if let Some(listeners) = listeners {
                    for (_id, listener) in listeners {
                        listener.borrow_mut()(&*msg)
                    }
                }
            }
            RuntimeMessageKind::Handle { key, f } => {
                let me = self.inner.borrow();
                let node = me.nodes[key].clone();
                drop(me);
                f(&mut *node.borrow_mut().object);
            }
            RuntimeMessageKind::Listen {
                id,
                key,
                type_id,
                f,
                listen_f,
                listener,
            } => {
                self.inner
                    .borrow_mut()
                    .listeners
                    .insert((key, type_id), vec![(id, f)]);

                let node = self.inner.borrow().nodes[key].clone();
                listen_f(&mut *node.borrow_mut().object);

                if let Some(handle) = listener {
                    self.inner.borrow_mut().nodes[key]
                        .borrow_mut()
                        .listener_guards
                        .push(ListenerGuard {
                            handle: BindHandle {
                                id,
                                type_id,
                                handle,
                            },
                        })
                }
            }
            RuntimeMessageKind::RemoveListener { id, key, type_id } => {
                if let Some(listeners) = self.inner.borrow_mut().listeners.get_mut(&(key, type_id))
                {
                    if let Some(idx) = listeners
                        .iter()
                        .position(|(listener_id, _)| *listener_id == id)
                    {
                        listeners.remove(idx);
                    }
                }
            }
            RuntimeMessageKind::Remove { key } => {
                self.inner.borrow_mut().nodes.remove(key);
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
