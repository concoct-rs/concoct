use crate::{Handle, Task};
use futures::{channel::mpsc, StreamExt};
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

pub(crate) struct Inner {
    pub(crate) tasks: SlotMap<DefaultKey, Rc<RefCell<dyn AnyTask>>>,
    pub(crate) listeners: HashMap<(DefaultKey, TypeId), Vec<Rc<RefCell<dyn FnMut(&dyn Any)>>>>,
    rx: mpsc::UnboundedReceiver<(DefaultKey, Box<dyn FnOnce(&mut dyn AnyTask)>)>,
}

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::default();
}

#[derive(Clone)]
pub struct Runtime {
    pub(crate) inner: Rc<RefCell<Inner>>,
    pub(crate) tx: mpsc::UnboundedSender<(DefaultKey, Box<dyn FnOnce(&mut dyn AnyTask)>)>,
}

impl Default for Runtime {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded();
        Self {
            inner: Rc::new(RefCell::new(Inner {
                tasks: SlotMap::new(),
                listeners: HashMap::new(),
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

    pub fn spawn<T>(&self, task: T) -> Handle<T>
    where
        T: Task + 'static,
    {
        let key = self
            .inner
            .borrow_mut()
            .tasks
            .insert(Rc::new(RefCell::new(task)));

        Handle::new(key)
    }

    pub async fn run(&self) {
        let mut me = self.inner.borrow_mut();
        if let Some((key, update)) = me.rx.next().await {
            let task = me.tasks[key].clone();
            drop(me);

            let mut task_ref = task.borrow_mut();
            update(&mut *task_ref);
            drop(task_ref);

            loop {
                let mut me = self.inner.borrow_mut();
                if let Ok(Some((key, update))) = me.rx.try_next() {
                    let task = me.tasks[key].clone();
                    drop(me);

                    let mut task_ref = task.borrow_mut();
                    update(&mut *task_ref);
                } else {
                    break;
                }
            }
        }
    }
}

pub(crate) trait AnyTask {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Task + 'static> AnyTask for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
