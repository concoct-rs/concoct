use crate::{node::Node, AnyView, Platform};
use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{cell::RefCell, rc::Rc};

thread_local! {
    static VIEW_CONTEXT: RefCell<Option<ViewContext>> = RefCell::default();
}

pub(crate) struct Inner {
    pub(crate) parent_key: DefaultKey,
    pub(crate) nodes: SlotMap<DefaultKey, Rc<RefCell<Node>>>,
    pub(crate) children: SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
    pub(crate) tracked: SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
    pub(crate) platform: Rc<dyn Platform>,
    pub(crate) is_done: bool,
}

#[derive(Clone)]
pub struct ViewContext {
    pub(crate) inner: Rc<RefCell<Inner>>,
}

impl ViewContext {
    pub fn new(platform: impl Platform + 'static) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                parent_key: Default::default(),
                nodes: Default::default(),
                children: Default::default(),
                tracked: Default::default(),
                platform: Rc::new(platform),
                is_done: false,
            })),
        }
    }

    pub fn current() -> Self {
        VIEW_CONTEXT
            .try_with(|cx| cx.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub fn try_current() -> Option<Self> {
        VIEW_CONTEXT
            .try_with(|cx| cx.borrow().as_ref().cloned())
            .unwrap()
    }

    pub fn enter(self) {
        VIEW_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self))
            .unwrap();
    }

    pub fn insert(&mut self, make_view: Box<dyn FnMut() -> Box<dyn AnyView>>) -> DefaultKey {
        let mut inner = self.inner.borrow_mut();
        let me = &mut *inner;

        let contexts = me.nodes[me.parent_key].borrow().contexts.clone();
        let node = Node {
            make_view,
            view: None,
            hooks: Rc::default(),
            contexts,
            on_drops: Rc::default(),
        };
        let key = me.nodes.insert(Rc::new(RefCell::new(node)));

        if let Some(children) = me.children.get_mut(me.parent_key) {
            children.push(key);
        } else {
            me.children.insert(me.parent_key, vec![key]);
        }

        key
    }
}
