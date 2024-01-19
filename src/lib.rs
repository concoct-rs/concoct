use slotmap::{DefaultKey, SlotMap};
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::task::{Poll, Waker};
use std::{cell::RefCell, mem, rc::Rc};

pub mod body;
pub use self::body::Body;
use body::Empty;

pub mod hook;

pub mod view;
pub use self::view::View;

pub mod web;
use web::WebRoot;

#[derive(Default)]
struct ScopeInner {
    contexts: HashMap<TypeId, Rc<dyn Any>>,
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
    waker: Option<Waker>,
    contexts: HashMap<TypeId, Rc<dyn Any>>,
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

    pub async fn rebuild(&self) {
        futures::future::poll_fn(|cx| {
            let mut inner = self.inner.borrow_mut();
            inner.waker = Some(cx.waker().clone());

            if let Some(key) = inner.pending.pop_front() {
                let raw = inner.nodes[key];
                drop(inner);

                let pending = unsafe { &mut *raw };
                pending.build();
            }

            Poll::Pending
        })
        .await
    }
}

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

pub struct Node<V, B, F> {
    view: V,
    body: Option<B>,
    builder: F,
    scope: Scope,
    key: Option<DefaultKey>,
}

pub trait Tree: 'static {
    fn build(&mut self);

    fn rebuild(&mut self, last: &mut dyn Any);
}

macro_rules! impl_tree_for_tuple {
    ($($t:tt : $idx:tt),*) => {
        impl<$($t: Tree),*> Tree for ($($t),*) {
            fn build(&mut self) {
               $(
                    self.$idx.build();
               )*
            }

            fn rebuild(&mut self, last: &mut dyn Any) {
                if let Some(last) = last.downcast_mut::<Self>() {
                    $(
                        self.$idx.rebuild(&mut last.$idx);
                    )*
                }
            }
        }

    };
}

impl_tree_for_tuple!(V1: 0, V2: 1);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8, V10: 9);

impl Tree for Empty {
    fn build(&mut self) {}

    fn rebuild(&mut self, _last: &mut dyn Any) {}
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

        if let Some(key) = self.key {
            let mut scope = self.scope.inner.borrow_mut();
            scope.contexts = cx_ref.contexts.clone();
            drop(scope);

            cx_ref.node = Some(key);
            let _parent_scope = mem::replace(&mut cx_ref.scope, Some(self.scope.clone()));
            drop(cx_ref);

            let view = unsafe { mem::transmute(&self.view) };
            let body = (self.builder)(view);
            let mut body = mem::replace(&mut self.body, Some(body)).unwrap();
            self.body.as_mut().unwrap().rebuild(&mut body);

            let mut cx_ref = cx.inner.borrow_mut();
            let mut scope = self.scope.inner.borrow_mut();
            cx_ref.contexts = scope.contexts.clone();
            scope.hook_idx = 0;
        } else {
            let key = cx_ref.nodes.insert(self as _);
            self.key = Some(key);

            let mut scope = self.scope.inner.borrow_mut();
            scope.contexts = cx_ref.contexts.clone();
            drop(scope);

            cx_ref.node = Some(key);
            cx_ref.scope = Some(self.scope.clone());
            drop(cx_ref);

            let view = unsafe { mem::transmute(&self.view) };
            let body = (self.builder)(view);

            let parent_contexts = {
                let mut cx_ref = cx.inner.borrow_mut();
                let mut scope = self.scope.inner.borrow_mut();
                scope.hook_idx = 0;
                mem::replace(&mut cx_ref.contexts, scope.contexts.clone())
            };

            self.body = Some(body);
            self.body.as_mut().unwrap().build();

            let mut cx_ref = cx.inner.borrow_mut();
            cx_ref.contexts = parent_contexts;
        }
    }

    fn rebuild(&mut self, last: &mut dyn Any) {
        if let Some(last) = last.downcast_mut::<Self>() {
            let key = last.key.unwrap();
            self.key = Some(key);
            self.scope = last.scope.clone();

            let cx = Context::current();
            let mut cx_ref = cx.inner.borrow_mut();
            cx_ref.node = Some(key);
            cx_ref.scope = Some(self.scope.clone());
            drop(cx_ref);

            let view = unsafe { mem::transmute(&self.view) };
            let body = (self.builder)(view);
            self.body = Some(body);
            self.body.as_mut().unwrap().rebuild(&mut last.body);

            self.scope.inner.borrow_mut().hook_idx = 0;
        }
    }
}

pub async fn run(view: impl View) {
    let cx = Context::default();
    cx.enter();

    let mut tree = WebRoot {
        body: Rc::new(view),
    }
    .into_tree();
    tree.build();

    loop {
        cx.rebuild().await
    }
}

pub struct TextViewContext {
    view: RefCell<Box<dyn FnMut(Cow<'static, str>)>>,
}

impl TextViewContext {
    pub fn new(view: impl FnMut(Cow<'static, str>) + 'static) -> Self {
        Self {
            view: RefCell::new(Box::new(view)),
        }
    }
}
