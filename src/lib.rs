use futures::channel::mpsc;
use futures::executor::LocalPool;
use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{any::Any, cell::RefCell, collections::HashSet, rc::Rc};

mod any_view;
pub use any_view::AnyView;

mod composable;
pub use composable::{Child, View};

mod composition;
pub use composition::Composition;

mod into_view;
pub use self::into_view::IntoView;

mod node;
use node::Node;

mod use_ref;
pub use use_ref::{use_ref, Ref, RefMut, UseRef};

mod use_future;
pub use use_future::use_future;

mod use_state;
pub use use_state::{use_state, UseState};

#[cfg(feature = "html")]
pub mod html;

#[cfg(feature = "web")]
pub mod web;

#[derive(Default)]
struct GlobalContext {
    values: SlotMap<DefaultKey, Rc<RefCell<dyn Any>>>,
    dirty: HashSet<DefaultKey>,
}

thread_local! {
    static GLOBAL_CONTEXT: RefCell<GlobalContext> = RefCell::default();
}

#[derive(Clone)]
struct TaskContext {
    local_pool: Rc<RefCell<LocalPool>>,
    tx: mpsc::UnboundedSender<Box<dyn Any>>,
}

thread_local! {
    static TASK_CONTEXT: RefCell<Option<TaskContext>> = RefCell::default();

    static BUILD_CONTEXT: RefCell<Option<Rc<RefCell<BuildContext>>>> = RefCell::default();
}

pub trait Platform {
    fn from_str(&mut self, s: &str) -> Box<dyn AnyView>;
}

impl Platform for () {
    fn from_str(&mut self, _s: &str) -> Box<dyn AnyView> {
        Box::new(())
    }
}

pub struct BuildContext {
    parent_key: DefaultKey,
    nodes: SlotMap<DefaultKey, Rc<RefCell<Node>>>,
    children: SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
    tracked: SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
    platform: Box<dyn Platform>,
}

impl BuildContext {
    pub fn new(platform: impl Platform + 'static) -> Self {
        Self {
            parent_key: Default::default(),
            nodes: Default::default(),
            children: Default::default(),
            tracked: Default::default(),
            platform: Box::new(platform),
        }
    }

    pub fn insert(&mut self, make_composable: Box<dyn FnMut() -> Box<dyn AnyView>>) -> DefaultKey {
        let node = Node {
            make_composable,
            composable: None,
            hooks: Rc::default(),
        };
        let key = self.nodes.insert(Rc::new(RefCell::new(node)));

        if let Some(children) = self.children.get_mut(self.parent_key) {
            children.push(key);
        } else {
            self.children.insert(self.parent_key, vec![key]);
        }

        key
    }
}

struct Scope {
    hooks: Rc<RefCell<Vec<Rc<RefCell<dyn Any>>>>>,
    idx: usize,
}

#[derive(Clone)]
struct LocalContext {
    scope: Rc<RefCell<Scope>>,
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
