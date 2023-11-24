use core::fmt;
use futures::Future;
use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{
    any::Any,
    cell::{Ref, RefCell},
    marker::PhantomData,
    mem::{self},
    ops::{Add, AddAssign},
    rc::Rc,
};
use tokio::{sync::mpsc, task::LocalSet};

mod composable;
pub use composable::{Composable, Debugger};

mod composition;
pub use composition::Composition;

mod node;
pub use node::Node;

#[derive(Default)]
struct GlobalContext {
    values: SlotMap<DefaultKey, Rc<RefCell<dyn Any>>>,
}

thread_local! {
    static GLOBAL_CONTEXT: RefCell<GlobalContext> = RefCell::default();
}

#[derive(Clone)]
struct TaskContext {
    local_set: Rc<RefCell<LocalSet>>,
    tx: mpsc::UnboundedSender<Box<dyn Any>>,
}

thread_local! {
    static TASK_CONTEXT: RefCell<Option<TaskContext>> = RefCell::default();
}

pub struct BuildContext<'a> {
    parent_key: DefaultKey,
    nodes: &'a mut SlotMap<DefaultKey, Node>,
    children: &'a mut SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
}

impl<'a> BuildContext<'a> {
    pub fn insert(&mut self, make_composable: Box<dyn FnMut() -> Box<dyn AnyComposable>>) {
        let node = Node {
            make_composable,
            composable: None,
            state: None,
            hooks: Rc::default(),
        };
        let key = self.nodes.insert(node);

        if let Some(children) = self.children.get_mut(self.parent_key) {
            children.push(key);
        } else {
            self.children.insert(self.parent_key, vec![key]);
        }
    }
}

pub trait AnyComposable {
    fn any_build(&mut self, cx: &mut BuildContext) -> Box<dyn Any>;

    fn any_rebuild(&mut self, state: &mut dyn Any);
}

impl<C: Composable> AnyComposable for C {
    fn any_build(&mut self, cx: &mut BuildContext) -> Box<dyn Any> {
        Box::new(self.build(cx))
    }

    fn any_rebuild(&mut self, state: &mut dyn Any) {
        self.rebuild(state.downcast_mut().unwrap())
    }
}

struct Inner {
    hooks: Rc<RefCell<Vec<Rc<RefCell<dyn Any>>>>>,
    idx: usize,
}

#[derive(Clone)]
pub struct LocalContext {
    inner: Rc<RefCell<Inner>>,
}

thread_local! {
    static LOCAL_CONTEXT: RefCell<Option<LocalContext>> = RefCell::default();
}

impl LocalContext {
    pub fn current() -> Self {
        LOCAL_CONTEXT
            .try_with(|cx| cx.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub fn enter(self) {
        LOCAL_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self))
            .unwrap()
    }
}

pub fn use_hook<T: 'static>(make_value: impl FnOnce() -> T) -> Rc<RefCell<dyn Any>> {
    let cx = LocalContext::current();
    let mut inner = cx.inner.borrow_mut();
    let mut hooks = inner.hooks.borrow_mut();

    let value = if let Some(hook) = hooks.get(inner.idx) {
        let value = hook.clone();

        value
    } else {
        hooks.push(Rc::new(RefCell::new(make_value())));
        hooks.last().as_deref().unwrap().clone()
    };

    drop(hooks);
    inner.idx += 1;

    value
}

pub struct State<T> {
    key: DefaultKey,
    _marker: PhantomData<T>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for State<T> {}

impl<T: 'static> State<T> {
    pub fn get(self) -> Ref<'static, T> {
        let rc = GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow_mut().values[self.key].clone())
            .unwrap();
        let output: Ref<'_, T> = Ref::map(rc.borrow(), |value| value.downcast_ref().unwrap());
        unsafe { mem::transmute(output) }
    }

    pub fn set(&self, value: T) {
        GLOBAL_CONTEXT
            .try_with(|cx| {
                *cx.borrow_mut().values[self.key]
                    .borrow_mut()
                    .downcast_mut()
                    .unwrap() = value
            })
            .unwrap();

        TASK_CONTEXT
            .try_with(|cx| {
                let guard = cx.borrow_mut();
                let cx = guard.as_ref().unwrap();
                let tx = cx.tx.clone();
                cx.local_set.borrow_mut().spawn_local(async move {
                    tx.send(Box::new(())).unwrap();
                });
            })
            .unwrap();
    }

    pub fn cloned(self) -> T
    where
        T: Clone,
    {
        self.get().clone()
    }
}

impl<T> fmt::Debug for State<T>
where
    T: fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl<T> AddAssign<T> for State<T>
where
    T: Add<Output = T> + Clone + 'static,
{
    fn add_assign(&mut self, rhs: T) {
        self.set(self.cloned() + rhs)
    }
}

pub fn use_state<T: 'static>(make_value: impl FnOnce() -> T) -> State<T> {
    let rc = use_hook(|| {
        GLOBAL_CONTEXT
            .try_with(|cx| {
                cx.borrow_mut()
                    .values
                    .insert(Rc::new(RefCell::new(make_value())))
            })
            .unwrap()
    });
    let guard = rc.borrow();
    let key: &DefaultKey = guard.downcast_ref().unwrap();

    State {
        key: *key,
        _marker: PhantomData,
    }
}

pub fn use_future<F: Future + 'static>(f: impl FnOnce() -> F) {
    use_hook(|| {
        let future = f();
        TASK_CONTEXT.try_with(|cx| {
            let guard = cx.borrow_mut();
            let cx = guard.as_ref().unwrap();
            let tx = cx.tx.clone();
            cx.local_set.borrow_mut().spawn_local(async move {
                future.await;
                tx.send(Box::new(())).unwrap();
            });
        })
    });
}
