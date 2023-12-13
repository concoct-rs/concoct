use crate::{Context, Handle, Object};
use alloc::rc::Rc;
use core::{
    any::{Any, TypeId},
    cell::RefCell,
    pin::Pin, hash::BuildHasherDefault,
    future::Future
};
use hashbrown::HashMap;
use rustc_hash::FxHasher;
use slotmap::{DefaultKey, SlotMap};

pub enum RuntimeMessage {
    Signal {
        key: DefaultKey,
        msg: Box<dyn Any>,
    },
    Handle {
        key: DefaultKey,
        f: Box<dyn FnOnce(&mut dyn AnyTask)>,
    },
}

pub(crate) struct Inner {
    pub(crate) tasks: SlotMap<DefaultKey, Rc<RefCell<dyn AnyTask>>>,
    pub(crate) listeners:
        HashMap<(DefaultKey, TypeId), Vec<Rc<RefCell<dyn FnMut(&dyn Any)>>>, BuildHasherDefault<FxHasher>>,
    pub(crate) channel: Box<dyn Channel>,
}

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::default();
}

#[derive(Clone)]
pub struct Runtime {
    pub(crate) inner: Rc<RefCell<Inner>>,
}

cfg_futures!(
    impl Default for Runtime {
        fn default() -> Self {
            let (tx, rx) = futures::channel::mpsc::unbounded();
            Self::new(Box::new(Mpsc {
                tx,rx
            }))
        }
    }
);

impl Runtime {
    pub fn new(channel: Box<dyn Channel>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                tasks: SlotMap::new(),
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

    pub fn spawn<T>(&self, task: T) -> Handle<T>
    where
        T: Object + 'static,
    {
        let key = self
            .inner
            .borrow_mut()
            .tasks
            .insert(Rc::new(RefCell::new(task)));

        let task = self.inner.borrow().tasks[key].clone();
        task.borrow_mut().start_any(key);

        Handle::new(key)
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
        match msg {
            RuntimeMessage::Signal { key, msg } => {
                if let Some(listeners) = Runtime::current()
                    .inner
                    .borrow()
                    .listeners
                    .get(&(key, (&*msg).type_id()))
                    .clone()
                {
                    for listener in listeners {
                        listener.borrow_mut()(&*msg)
                    }
                }
            }
            RuntimeMessage::Handle { key, f } => {
                let me = self.inner.borrow();
                let task = me.tasks[key].clone();
                drop(me);

                let mut task_ref = task.borrow_mut();
                f(&mut *task_ref);
            }
        }
    }
}

pub trait AnyTask {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn start_any(&mut self, key: DefaultKey);
}

impl<T: Object + 'static> AnyTask for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn start_any(&mut self, key: DefaultKey) {
        self.start(Context::new(key))
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

pub trait Channel {
    fn send(&mut self, msg: RuntimeMessage);

    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<RuntimeMessage>> + '_>>;

    fn try_next(&mut self) -> Option<RuntimeMessage>;
}

cfg_futures!(
    pub struct Mpsc {
        pub tx: futures::channel::mpsc::UnboundedSender<RuntimeMessage>,
        pub rx: futures::channel::mpsc::UnboundedReceiver<RuntimeMessage>,
    }

    impl Channel for Mpsc {
        fn send(&mut self, msg: RuntimeMessage) {
            self.tx.unbounded_send(msg).unwrap();
        }

        fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<RuntimeMessage>> + '_>> {
            use futures::StreamExt;
            Box::pin(async move {
                self.rx.next().await
            })
        }

        fn try_next(&mut self) -> Option<RuntimeMessage> {
            self.rx.try_next().ok().flatten()
        }
    }
);
