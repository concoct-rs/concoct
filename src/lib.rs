use std::any::Any;
use std::collections::VecDeque;
use std::{cell::RefCell, mem, rc::Rc};

pub mod body;
pub use self::body::Body;
use body::Empty;
use slotmap::{DefaultKey, SlotMap};

pub mod hook;

pub mod view;
pub use self::view::View;

#[derive(Default)]
struct ScopeInner {
    hooks: Vec<Rc<dyn Any>>,
    hook_idx: usize,
}

#[derive(Clone, Default)]
pub struct Scope {
    inner: Rc<RefCell<ScopeInner>>,
}

#[derive(Default)]
struct ContextInner {
    node: Option<DefaultKey>,
    pending: VecDeque<DefaultKey>,
    scope: Option<Scope>,
    nodes: SlotMap<DefaultKey, *mut dyn Tree>,
}

#[derive(Clone, Default)]
pub struct Context {
    inner: Rc<RefCell<ContextInner>>,
}

impl Context {
    pub fn enter(&self) {
        CONTEXT
            .try_with(|cell| *cell.borrow_mut() = Some(self.clone()))
            .unwrap();
    }

    pub fn current() -> Self {
        CONTEXT
            .try_with(|cell| cell.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub fn rebuild(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(key) = inner.pending.pop_front() {
            let raw = inner.nodes[key];
            drop(inner);

            let pending = unsafe { &mut *raw };
            pending.build();
        }
    }
}

pub fn request_update() {
    let cx = Context::current();
    let cx = &mut *cx.inner.borrow_mut();
    cx.pending.push_back(cx.node.unwrap());
}

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

pub struct Node<V, B, F> {
    view: V,
    body: Option<B>,
    builder: F,
    scope: Scope,
}

pub trait Tree {
    fn build(&mut self);
}

impl<T1: Tree, T2: Tree> Tree for (T1, T2) {
    fn build(&mut self) {
        self.0.build();
        self.1.build();
    }
}

impl Tree for Empty {
    fn build(&mut self) {}
}

impl<V, B, F> Tree for Node<V, B, F>
where
    V: View,
    B: Tree + 'static,
    F: FnMut(&'static V) -> B + 'static,
{
    fn build(&mut self) {
        let cx = Context::current();
        let mut cx_ref = cx.inner.borrow_mut();

        let key = cx_ref.nodes.insert(self as _);
        cx_ref.node = Some(key);

        cx_ref.scope = Some(self.scope.clone());
        drop(cx_ref);

        let view = unsafe { mem::transmute(&self.view) };
        let body = (self.builder)(view);
        self.body = Some(body);
        self.body.as_mut().unwrap().build();

        self.scope.inner.borrow_mut().hook_idx = 0;
    }
}
