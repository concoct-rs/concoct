use crate::{Handle, Object, object::AnyObject};
use alloc::rc::Rc;
use core::{
    any::{Any, TypeId},
    cell::RefCell,
    future::Future,
    hash::BuildHasherDefault,
    pin::Pin,
};
use hashbrown::HashMap;
use rustc_hash::FxHasher;
use slotmap::{DefaultKey, SlotMap};


pub struct RuntimeMessage( pub(crate)RuntimeMessageKind);

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
}

pub(crate) struct Inner {
    pub(crate) objects: SlotMap<DefaultKey, Rc<RefCell<dyn AnyObject>>>,
    pub(crate) listeners: HashMap<
        (DefaultKey, TypeId),
        Vec<Rc<RefCell<dyn FnMut(&dyn Any)>>>,
        BuildHasherDefault<FxHasher>,
    >,
    pub(crate) channel: Box<dyn Executor>,
}

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::default();
}

/// Local reactive object runtime.
///
/// This executes tasks in a thread-per-core fashion.
#[derive(Clone)]
pub struct Runtime {
    pub(crate) inner: Rc<RefCell<Inner>>,
}

cfg_futures!(
    impl Default for Runtime {
        fn default() -> Self {
            let (tx, rx) = futures::channel::mpsc::unbounded();
            Self::new(Box::new(LocalExecutor {
                tx,
                rx,
                local_set: tokio::task::LocalSet::new()
            }))
        }
    }
);

impl Runtime {
    pub fn new(channel: Box<dyn Executor>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                objects: SlotMap::new(),
                listeners: HashMap::with_hasher(BuildHasherDefault::default()),
                channel,
            })),
        }
    }

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
        let handle = Handle::new(key);
        object.borrow_mut().started_any(handle.guard.clone());
        handle
    }

    pub async fn run(&self) {
        let mut me = self.inner.borrow_mut();
        if let Some(msg) = me.channel.next().await {
            drop(me);
            self.run_inner(msg);

            self.try_run();
        }
    }

    pub fn try_run(&self) {
        loop {
            let mut me = self.inner.borrow_mut();
            if let Some(msg) = me.channel.try_next() {
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

pub trait Executor {
    fn send(&mut self, msg: RuntimeMessage);

    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<RuntimeMessage>> + '_>>;

    fn try_next(&mut self) -> Option<RuntimeMessage>;
}

cfg_futures!(
    pub struct LocalExecutor {
        pub tx: futures::channel::mpsc::UnboundedSender<RuntimeMessage>,
        pub rx: futures::channel::mpsc::UnboundedReceiver<RuntimeMessage>,
        pub local_set: tokio::task::LocalSet
    }

    impl Executor for LocalExecutor {
        fn send(&mut self, msg: RuntimeMessage) {
            self.tx.unbounded_send(msg).unwrap();
        }

        fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<RuntimeMessage>> + '_>> {
            use futures::StreamExt;
            Box::pin(async move {
                let _ = futures::poll!(&mut self.local_set);
                self.rx.next().await
            })
        }

        fn try_next(&mut self) -> Option<RuntimeMessage> {
            self.rx.try_next().ok().flatten()
        }
    }
);
