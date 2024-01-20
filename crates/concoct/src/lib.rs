use rustc_hash::FxHasher;
use slotmap::{DefaultKey, SlotMap};
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::task::{Poll, Waker};
use std::{cell::RefCell, rc::Rc};

pub mod hook;

mod tree;
pub(crate) use tree::Node;
pub use tree::Tree;

mod view_builder;
pub use self::view_builder::ViewBuilder;

pub mod view;
pub use self::view::View;
use view::Empty;

#[derive(Default)]
struct ScopeInner {
    contexts: HashMap<TypeId, Rc<dyn Any>>,
    hooks: Vec<Rc<dyn Any>>,
    hook_idx: usize,
    droppers: Vec<Box<dyn FnMut()>>,
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

impl Tree for Empty {
    fn build(&mut self) {}

    fn rebuild(&mut self, _last: &mut dyn Any) {}

    fn remove(&mut self) {}
}

pub async fn run(view: impl ViewBuilder) {
    let cx = Context::default();
    cx.enter();

    let mut tree = view.into_tree();
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

pub fn memo<B>(input: impl Hash, body: B) -> Memo<B> {
    let mut hasher = FxHasher::default();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    Memo { hash, body }
}

pub struct Memo<B> {
    hash: u64,
    body: B,
}

impl<B: View> View for Memo<B> {
    fn into_tree(self) -> impl Tree {
        Memo {
            hash: self.hash,
            body: self.body.into_tree(),
        }
    }
}

impl<T: Tree> Tree for Memo<T> {
    fn build(&mut self) {
        self.body.build()
    }

    fn rebuild(&mut self, last: &mut dyn Any) {
        let last = last.downcast_mut::<Self>().unwrap();
        if self.hash != last.hash {
            self.body.rebuild(&mut last.body)
        }
    }

    fn remove(&mut self) {
        self.body.remove()
    }
}
