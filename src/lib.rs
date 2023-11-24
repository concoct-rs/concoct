use core::fmt;
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::Any,
    cell::{Ref, RefCell},
    marker::PhantomData,
    rc::Rc,
};

mod composable;
pub use composable::{Composable, Debugger};

mod composition;
pub use composition::Composition;

mod node;
pub use node::Node;

pub struct BuildContext<'a> {
    nodes: &'a mut SlotMap<DefaultKey, Node>,
}

impl<'a> BuildContext<'a> {
    pub fn insert(&mut self, make_composable: Box<dyn FnMut() -> Box<dyn AnyComposable>>) {
        let _node = Node {
            make_composable,
            composable: None,
            state: None,
            hooks: Rc::default(),
        };
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
    value: Rc<RefCell<dyn Any>>,
    _marker: PhantomData<T>,
}

impl<T: 'static> State<T> {
    pub fn get(&self) -> Ref<T> {
        Ref::map(self.value.borrow(), |value| value.downcast_ref().unwrap())
    }

    pub fn cloned(&self) -> T
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
        self.value.borrow().downcast_ref::<T>().unwrap().fmt(f)
    }
}

pub fn use_state<T: 'static>(make_value: impl FnOnce() -> T) -> (State<T>, impl Fn(T)) {
    let value = use_hook(make_value);
    let value_clone = value.clone();
    (
        State {
            value,
            _marker: PhantomData,
        },
        move |new_value| {
            *value_clone.borrow_mut().downcast_mut().unwrap() = new_value;
        },
    )
}
