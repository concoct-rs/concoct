use crate::{Scope, View};
use slotmap::{DefaultKey, SlotMap};
use std::{cell::RefCell, collections::HashSet, mem, rc::Rc};

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = RefCell::new(None);
}

#[derive(Default)]
pub(crate) struct Inner {
    scopes: SlotMap<DefaultKey, Option<Scope>>,
    pending: Vec<(Box<dyn View>, Option<DefaultKey>)>,
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

    pub fn try_current() -> Option<Self> {
        CURRENT
            .try_with(|rt| rt.borrow().as_ref().cloned())
            .unwrap()
    }

    pub fn spawn(&self, view: impl View + 'static) {
        let parent_key = Scope::try_current().map(|scope| scope.inner.borrow().key);
        self.inner
            .borrow_mut()
            .pending
            .push((Box::new(view), parent_key));
    }

    pub fn update(&self, key: DefaultKey) {
        let signals = self.inner.borrow_mut().signals[key].clone();
        for scope_key in signals {
            let mut inner = self.inner.borrow_mut();
            let scope = inner.scopes[scope_key].as_mut().unwrap().clone();
            drop(inner);

            scope.run();
        }
    }

    pub fn poll(&self) {
        let mut me = self.inner.borrow_mut();
        if !me.pending.is_empty() {
            let pending = mem::take(&mut me.pending);
            drop(me);

            for (view, parent_key) in pending {
                let mut me = self.inner.borrow_mut();
                let key = me.scopes.insert(None);

                let contexts = parent_key
                    .map(|parent_key| {
                        me.scopes[parent_key]
                            .as_ref()
                            .unwrap()
                            .inner
                            .borrow()
                            .contexts
                            .clone()
                    })
                    .unwrap_or_default();

                drop(me);

                let scope = Scope::new(key, parent_key, contexts, view);
                let key = scope.inner.borrow().key;
                scope.run();

                let mut me = self.inner.borrow_mut();
                me.scopes[key] = Some(scope);
            }
        }
    }
}
