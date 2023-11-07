use crate::{Scope, View};
use slotmap::{DefaultKey, SlotMap};
use std::{cell::RefCell, collections::HashSet, mem, rc::Rc};

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::new(None);
}

#[derive(Default)]
pub(crate) struct Inner {
    scopes: SlotMap<DefaultKey, Option<Scope>>,
    pending: Vec<Box<dyn FnOnce() -> Scope>>,
    pub(crate) signals: SlotMap<DefaultKey, HashSet<DefaultKey>>,
}

#[derive(Clone, Default)]
pub struct Runtime {
    pub(crate) inner: Rc<RefCell<Inner>>,
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

    pub fn spawn<V>(&self, component: impl FnMut() -> V + 'static)
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
        let signals = self.inner.borrow_mut().signals[key].clone();

        for scope_key in signals {
            let mut inner = self.inner.borrow_mut();
            let scope = inner.scopes[scope_key].as_mut().unwrap();
            scope.clone().enter();
            let component = scope.inner.borrow_mut().component.clone();
            drop(inner);
            let mut view = component.borrow_mut()();
            view.view();
        }
    }

    pub fn poll(&self) {
        let mut me = self.inner.borrow_mut();
        if !me.pending.is_empty() {
            let pending = mem::take(&mut me.pending);
            drop(me);

            for f in pending {
                let scope = f();
                let key = scope.inner.borrow().key;
                let inner = scope.inner.borrow_mut();
                scope.clone().enter();

                let component = inner.component.clone();
                drop(inner);
                let mut view = component.borrow_mut()();
                view.view();

                let mut me = self.inner.borrow_mut();
                me.scopes[key] = Some(scope);
            }
        }
    }
}
