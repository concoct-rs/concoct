use crate::{View, STORE};
use generational_box::{GenerationalBox, Owner};
use slotmap::DefaultKey;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    mem,
    rc::Rc,
};

pub(crate) struct Inner {
    pub owner: Owner,
    pub component: Rc<RefCell<dyn View>>,
    pub key: DefaultKey,
    pub(crate) hooks: Vec<GenerationalBox<Box<dyn Any>>>,
    pub(crate) hook_idx: RefCell<usize>,
    parent_key: Option<DefaultKey>,
    pub(crate) contexts: HashMap<TypeId, Rc<dyn Any>>,
}

thread_local! {
    static CURRENT:RefCell<Option<Scope>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct Scope {
    pub(crate) inner: Rc<RefCell<Inner>>,
}

impl Scope {
    pub fn new(
        key: DefaultKey,
        parent_key: Option<DefaultKey>,
        contexts: HashMap<TypeId, Rc<dyn Any>>,
        view: impl View + 'static,
    ) -> Self {
        let me = Self {
            inner: Rc::new(RefCell::new(Inner {
                owner: STORE.try_with(|store| store.owner()).unwrap(),
                component: Rc::new(RefCell::new(view)),
                key,
                hook_idx: RefCell::new(0),
                hooks: Vec::new(),
                parent_key,
                contexts,
            })),
        };
        me
    }

    pub fn current() -> Self {
        CURRENT
            .try_with(|current| current.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub fn try_current() -> Option<Self> {
        CURRENT
            .try_with(|current| current.borrow().as_ref().cloned())
            .unwrap()
    }

    pub fn enter(self) -> Option<Self> {
        CURRENT
            .try_with(|current| mem::replace(&mut *current.borrow_mut(), Some(self)))
            .unwrap()
    }

    pub fn run(&self) {
        self.clone().enter();
        let inner = self.inner.borrow_mut();
        let component = inner.component.clone();
        *inner.hook_idx.borrow_mut() = 0;
        drop(inner);

        let mut view = component.borrow_mut();
        let _node = view.view();
    }
}
