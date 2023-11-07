use crate::{Scope, View};
use slotmap::{DefaultKey, SlotMap};
use std::{cell::RefCell, mem, rc::Rc};

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::new(None);
}

#[derive(Default)]
struct Inner {
    scopes: SlotMap<DefaultKey, Option<Scope>>,
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
        let key = self.inner.borrow_mut().scopes.insert(None);
        self.spawn_with_scope(Box::new(move || Scope::new(key, component)))
    }

    pub fn spawn_with_scope(&self, component: Box<dyn FnOnce() -> Scope>) {
        self.inner.borrow_mut().pending.push(component);
    }

    pub fn update(&self, key: DefaultKey) {
        self.inner.borrow_mut().scopes[key]
            .as_ref()
            .unwrap()
            .inner
            .borrow_mut()
            .view
            .as_mut()
            .unwrap()
            .view();
    }

    pub fn poll(&self) {
        let mut me = self.inner.borrow_mut();
        if !me.pending.is_empty() {
            let pending = mem::take(&mut me.pending);
            drop(me);

            for f in pending {
                let scope = f();
                let key = scope.inner.borrow().key;
                let mut inner = scope.inner.borrow_mut();
                let view = inner.view.as_mut().unwrap();
                view.view();
                drop(inner);

                let mut me = self.inner.borrow_mut();
                me.scopes[key] = Some(scope);
            }
        }
    }
}
