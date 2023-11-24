use core::fmt;
use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{
    any::Any,
    cell::{Ref, RefCell},
    marker::PhantomData,
    mem::{self},
    ops::AddAssign,
    rc::Rc,
};

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
    let inner = cx.inner.borrow_mut();
    let mut hooks = inner.hooks.borrow_mut();

    if let Some(hook) = hooks.get(inner.idx) {
        hook.clone()
    } else {
        hooks.push(Rc::new(RefCell::new(make_value())));
        hooks.last().as_deref().unwrap().clone()
    }
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

impl<T: AddAssign + 'static> AddAssign<T> for State<T> {
    fn add_assign(&mut self, _rhs: T) {
        todo!()
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

pub fn use_future<F>(_f: impl FnOnce() -> F) {}
