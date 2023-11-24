use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{any::Any, cell::RefCell, rc::Rc};

pub struct BuildContext<'a> {
    nodes: &'a mut SlotMap<DefaultKey, Node>,
}

impl<'a> BuildContext<'a> {
    pub fn insert(&mut self, make_composable: Box<dyn FnMut() -> Box<dyn AnyComposable>>) {
        let node = Node {
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

impl<F: FnMut() -> C, C: Composable> Composable for F {
    type State = C::State;

    fn build(&mut self, cx: &mut BuildContext) -> Self::State {
        todo!()
    }

    fn rebuild(&mut self, state: &mut Self::State) {
        todo!()
    }
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

struct Node {
    make_composable: Box<dyn FnMut() -> Box<dyn AnyComposable>>,
    composable: Option<Box<dyn AnyComposable>>,
    state: Option<Box<dyn Any>>,
    hooks: Rc<RefCell<Vec<Rc<RefCell<dyn Any>>>>>,
}

pub struct Composition {
    nodes: SlotMap<DefaultKey, Node>,
    children: SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
    root: DefaultKey,
}

impl Composition {
    pub fn new<C>(content: fn() -> C) -> Self
    where
        C: Composable + 'static,
    {
        let mut composables = SlotMap::new();
        let make_composable = Box::new(move || {
            let composable: Box<dyn AnyComposable> = Box::new(content());
            composable
        });
        let node = Node {
            make_composable,
            composable: None,
            state: None,
            hooks: Rc::default(),
        };
        let root = composables.insert(node);

        Self {
            nodes: composables,
            children: SparseSecondaryMap::new(),
            root,
        }
    }

    pub fn build(&mut self) {
        let node = &mut self.nodes[self.root];

        let cx = LocalContext {
            inner: Rc::new(RefCell::new(Inner {
                hooks: node.hooks.clone(),
                idx: 0,
            })),
        };
        cx.enter();
        let mut composable = (node.make_composable)();

        let mut build_cx = BuildContext {
            nodes: &mut self.nodes,
        };
        let state = composable.any_build(&mut build_cx);

        let node = &mut self.nodes[self.root];
        node.composable = Some(composable);
        node.state = Some(state);
    }

    pub fn rebuild(&mut self) {
        let node = &mut self.nodes[self.root];

        let cx = LocalContext {
            inner: Rc::new(RefCell::new(Inner {
                hooks: node.hooks.clone(),
                idx: 0,
            })),
        };
        cx.enter();
        let mut composable = (node.make_composable)();
        let state = node.state.as_mut().unwrap();
        composable.any_rebuild(&mut **state);
        node.composable = Some(composable);
    }
}

struct Inner {
    hooks: Rc<RefCell<Vec<Rc<RefCell<dyn Any>>>>>,
    idx: usize,
}

thread_local! {
    static LOCAL_CONTEXT: RefCell<Option<LocalContext>> = RefCell::default();
}

#[derive(Clone)]
pub struct LocalContext {
    inner: Rc<RefCell<Inner>>,
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
