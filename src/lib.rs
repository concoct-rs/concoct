use futures::{channel::mpsc, StreamExt};
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
};

pub trait Task {
    fn spawn(self) -> Handle<Self>
    where
        Self: Sized + 'static,
    {
        Runtime::current().spawn(self)
    }
}

pub trait Handler<M>: Task {
    fn handle(&mut self, msg: M);
}

pub trait AnyTask {
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

struct Inner {
    tasks: SlotMap<DefaultKey, Rc<RefCell<dyn AnyTask>>>,
    listeners: HashMap<(DefaultKey, TypeId), Vec<Rc<RefCell<dyn FnMut(&dyn Any)>>>>,
    rx: mpsc::UnboundedReceiver<(DefaultKey, Box<dyn FnOnce(&mut dyn AnyTask)>)>,
}

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::default();
}

#[derive(Clone)]
pub struct Runtime {
    inner: Rc<RefCell<Inner>>,
    tx: mpsc::UnboundedSender<(DefaultKey, Box<dyn FnOnce(&mut dyn AnyTask)>)>,
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
        CURRENT.try_with(|cell| cell.borrow().clone()).unwrap()
    }

    pub fn enter(&self) {
        CURRENT
            .try_with(|cell| *cell.borrow_mut() = Some(self.clone()))
            .unwrap()
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

        Handle {
            key,
            _marker: PhantomData,
        }
    }

    pub async fn run(&self) {
        loop {
            let mut me = self.inner.borrow_mut();
            if let Some((key, update)) = me.rx.next().await {
                let task = me.tasks[key].clone();
                drop(me);

                let mut task_ref = task.borrow_mut();
                update(&mut *task_ref);
            }
        }
    }
}

struct Dropper {
    key: DefaultKey,
}

impl Drop for Dropper {
    fn drop(&mut self) {
        Runtime::current().inner.borrow_mut().tasks.remove(self.key);
    }
}

pub struct Handle<T> {
    key: DefaultKey,
    _marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<T> Handle<T> {
    pub fn send<M>(&self, msg: M)
    where
        T: Handler<M> + 'static,
        M: 'static,
    {
        let key = self.key;
        Runtime::current()
            .tx
            .unbounded_send((
                key,
                Box::new(move |any_task| {
                    if let Some(listeners) = Runtime::current()
                        .inner
                        .borrow()
                        .listeners
                        .get(&(key, msg.type_id()))
                        .clone()
                    {
                        for listener in listeners {
                            listener.borrow_mut()(&msg)
                        }
                    }

                    let task = any_task.as_any_mut().downcast_mut::<T>().unwrap();
                    task.handle(msg);
                }),
            ))
            .unwrap();
    }

    pub fn listen<M: 'static>(&self, mut f: impl FnMut(&M) + 'static) {
        Runtime::current().inner.borrow_mut().listeners.insert(
            (self.key, TypeId::of::<M>()),
            vec![Rc::new(RefCell::new(move |msg: &dyn Any| {
                f(msg.downcast_ref().unwrap())
            }))],
        );
    }

    pub fn bind<M, T2>(&self, other: &Handle<T2>)
    where
        M: Clone + 'static,
        T2: Task + Handler<M> + 'static,
    {
        let other = other.clone();

        self.listen(move |msg: &M| {
            other.send(msg.clone());
        });
    }
}
