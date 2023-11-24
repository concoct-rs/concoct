use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{any::Any, cell::RefCell, rc::Rc};
use tokio::sync::mpsc;

mod any_composable;
pub use any_composable::AnyComposable;

mod composable;
pub use composable::Composable;

mod composition;
pub use composition::Composition;

mod node;
pub use node::Node;

mod use_hook;
pub use use_hook::{use_hook, UseHook};

mod use_future;
pub use use_future::{use_future, UseFuture};

mod use_state;
pub use use_state::{use_state, UseState};

#[derive(Default)]
struct GlobalContext {
    values: SlotMap<DefaultKey, Rc<RefCell<dyn Any>>>,
}

thread_local! {
    static GLOBAL_CONTEXT: RefCell<GlobalContext> = RefCell::default();
}

#[derive(Clone)]
struct TaskContext {
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
