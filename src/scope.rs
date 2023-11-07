use crate::{View, STORE};
use generational_box::{GenerationalBox, Owner};
use slotmap::DefaultKey;
use std::{
    any::Any,
    cell::{RefCell, RefMut},
    mem,
    rc::Rc,
};

pub(crate) struct Inner {
    pub owner: Owner,
    pub component: Rc<RefCell<dyn FnMut() -> Box<dyn View>>>,
    pub key: DefaultKey,
    hooks: Vec<GenerationalBox<Box<dyn Any>>>,
    hook_idx: RefCell<usize>,
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
                hook_idx: RefCell::new(0),
                hooks: Vec::new(),
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

    pub fn use_hook<T: 'static>(&self, f: impl FnOnce() -> T) -> RefMut<T> {
        let me = self.inner.borrow_mut();
        let idx = *me.hook_idx.borrow();
        let any = if let Some(any) = me.hooks.get(idx) {
            *any
        } else {
            drop(me);
            let value = f();
            let mut me = self.inner.borrow_mut();
            let any: GenerationalBox<Box<dyn Any>> = me.owner.insert(Box::new(value));
            me.hooks.push(any);
            *me.hooks.last().unwrap()
        };

        let me = self.inner.borrow_mut();
        *me.hook_idx.borrow_mut() += 1;
        RefMut::map(any.write(), |value| value.downcast_mut().unwrap())
    }

    pub fn run(&self) {
        self.clone().enter();
        let inner = self.inner.borrow_mut();
        let component = inner.component.clone();
        *inner.hook_idx.borrow_mut() = 0;
        drop(inner);

        let mut view = component.borrow_mut()();
        view.view();
    }
}
