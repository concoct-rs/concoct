use crate::Tree;
use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    task::Waker,
    time::Instant,
};

#[derive(Default)]
pub(crate) struct ScopeInner {
    pub(crate) contexts: FxHashMap<TypeId, Rc<dyn Any>>,
    pub(crate) hooks: Vec<Rc<dyn Any>>,
    pub(crate) hook_idx: usize,
    pub(crate) droppers: Vec<Box<dyn FnMut()>>,
}

#[derive(Clone, Default)]
pub(crate) struct Scope {
    pub(crate) inner: Rc<RefCell<ScopeInner>>,
}

#[derive(Default)]
pub(crate) struct RuntimeInner {
    pub(crate) node: Option<DefaultKey>,
    pub(crate) pending: VecDeque<DefaultKey>,
    pub(crate) scope: Option<Scope>,
    pub(crate) nodes: SlotMap<DefaultKey, *mut dyn Tree>,
    pub(crate) waker: Option<Waker>,
    pub(crate) contexts: FxHashMap<TypeId, Rc<dyn Any>>,
    pub(crate) limit: Option<Instant>,
}

#[derive(Clone, Default)]
pub struct Runtime {
    pub(crate) inner: Rc<RefCell<RuntimeInner>>,
}

impl Runtime {
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
}

thread_local! {
    static CONTEXT: RefCell<Option<Runtime>> = RefCell::new(None);
}
