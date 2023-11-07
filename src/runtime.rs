use crate::{Scope, View};
use slotmap::{DefaultKey, SlotMap};
use std::{cell::RefCell, mem, rc::Rc};

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::new(None);
}

#[derive(Default)]
struct Inner {
    scopes: SlotMap<DefaultKey, Scope>,
    pending: Vec<Box<dyn FnOnce() -> Scope>>,
}

#[derive(Clone, Default)]
pub struct Runtime {
    inner: Rc<RefCell<Inner>>,
}

impl Runtime {
    pub fn enter(self) -> Option<Self> {
        CURRENT
            .try_with(|rt| mem::replace(&mut *rt.borrow_mut(), Some(self)))
            .unwrap()
    }

    pub fn current() -> Self {
        CURRENT
            .try_with(|rt| rt.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub fn spawn<V>(&self, component: impl FnOnce() -> V + 'static)
    where
        V: View + 'static,
    {
        self.inner
            .borrow_mut()
            .pending
            .push(Box::new(|| Scope::new(component)));
    }

    pub fn poll(&self) {
        let mut me = self.inner.borrow_mut();
        if !me.pending.is_empty() {
            for f in mem::take(&mut me.pending) {
                let scope = f();
                me.scopes.insert(scope);
            }
        }
    }
}
