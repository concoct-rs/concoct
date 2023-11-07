use crate::{View, STORE};
use generational_box::Owner;
use slotmap::DefaultKey;
use std::{cell::RefCell, mem, rc::Rc};

pub(crate) struct Inner {
    pub owner: Owner,
    pub component: Rc<RefCell<dyn FnMut() -> Box<dyn View>>>,
    pub key: DefaultKey,
}

thread_local! {
    static CURRENT:RefCell<Option<Scope>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct Scope {
    pub(crate) inner: Rc<RefCell<Inner>>,
}

impl Scope {
    pub fn new<V: View + 'static>(
        key: DefaultKey,
        mut component: impl FnMut() -> V + 'static,
    ) -> Self {
        let me = Self {
            inner: Rc::new(RefCell::new(Inner {
                owner: STORE.try_with(|store| store.owner()).unwrap(),
                component: Rc::new(RefCell::new(move || {
                    let view: Box<dyn View> = Box::new(component());
                    view
                })),
                key,
            })),
        };
        me
    }

    pub fn current() -> Self {
        CURRENT
            .try_with(|current| current.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub fn enter(self) -> Option<Self> {
        CURRENT
            .try_with(|current| mem::replace(&mut *current.borrow_mut(), Some(self)))
            .unwrap()
    }

    pub fn track(&self, _key: DefaultKey) {}
}
