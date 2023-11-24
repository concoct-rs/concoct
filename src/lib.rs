use slotmap::{DefaultKey, SlotMap};
use std::{any::Any, cell::RefCell, rc::Rc};

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

pub trait Composable {
    type State: 'static;

    fn build(&mut self, cx: &mut BuildContext) -> Self::State;

    fn rebuild(&mut self, state: &mut Self::State);
}

impl<F, C> Composable for F
where
    F: FnMut() -> C + Clone + 'static,
    C: Composable + 'static,
{
    type State = ();

    fn build(&mut self, cx: &mut BuildContext) -> Self::State {
        let mut f = self.clone();
        cx.insert(Box::new(move || Box::new(f())));
    }

    fn rebuild(&mut self, _state: &mut Self::State) {}
}

impl<A: Composable, B: Composable> Composable for (A, B) {
    type State = (A::State, B::State);

    fn build(&mut self, cx: &mut BuildContext) -> Self::State {
        ((self.0).build(cx), (self.1).build(cx))
    }

    fn rebuild(&mut self, state: &mut Self::State) {
        (self.0).rebuild(&mut state.0);
        (self.1).rebuild(&mut state.1);
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
