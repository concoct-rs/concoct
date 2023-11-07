use crate::{View, STORE};
use generational_box::Owner;
use std::{cell::RefCell, mem, rc::Rc};

pub(crate) struct Inner {
    pub owner: Owner,
    view: Option<Box<dyn View>>,
}

thread_local! {
    static CURRENT:RefCell<Option<Scope>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct Scope {
    pub (crate) inner: Rc<RefCell<Inner>>,
}

impl Scope {
    pub fn new<V: View + 'static>(component: impl FnOnce() -> V) -> Self {
        let me = Self {
            inner: Rc::new(RefCell::new(Inner {
                owner: STORE.try_with(|store| store.owner()).unwrap(),
                view: None,
            })),
        };
        me.clone().enter();

        let view = component();
        me.inner.borrow_mut().view = Some(Box::new(view));
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
}
