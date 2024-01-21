use crate::{macros::trace, Runtime, Scope, Tree, ViewBuilder};
use slotmap::{DefaultKey, Key};
use std::{
    any::{self, Any},
    mem,
};

pub struct Node<V, B, F> {
    pub(crate) view: V,
    pub(crate) body: Option<B>,
    pub(crate) builder: F,
    pub(crate) scope: Option<Scope>,
    pub(crate) key: Option<DefaultKey>,
}

impl<V, B, F> Node<V, B, F> {
    fn name(&self) -> &'static str {
        let name = any::type_name::<V>();
        name.split('<')
            .next()
            .unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or(name)
    }
}

impl<V, B, F> Tree for Node<V, B, F>
where
    V: ViewBuilder,
    B: Tree + 'static,
    F: FnMut(&'static V) -> B + 'static,
{
    unsafe fn build(&mut self) {
        let cx = Runtime::current();
        let mut cx_ref = cx.inner.borrow_mut();

        if let Some(key) = self.key {
            let mut scope = self.scope.as_ref().unwrap().inner.borrow_mut();
            for (name, value) in cx_ref.contexts.iter() {
                if !scope.contexts.contains_key(name) {
                    scope.contexts.insert(*name, value.clone());
                }
            }
            drop(scope);

            cx_ref.node = Some(key);
            cx_ref.scope = Some(self.scope.clone().unwrap());
            drop(cx_ref);

            trace!("rebuilding from {:?}: {}", key.data(), self.name());

            let view = unsafe { mem::transmute(&self.view) };
            let body = (self.builder)(view);

            let parent_contexts = {
                let mut cx_ref = cx.inner.borrow_mut();
                let mut scope = self.scope.as_ref().unwrap().inner.borrow_mut();
                scope.hook_idx = 0;
                mem::replace(&mut cx_ref.contexts, scope.contexts.clone())
            };

            let mut last_body = mem::replace(&mut self.body, Some(body)).unwrap();
            self.body.as_mut().unwrap().rebuild(&mut last_body, true);

            let mut cx_ref = cx.inner.borrow_mut();
            cx_ref.contexts = parent_contexts;
        } else {
            let key = cx_ref.nodes.insert(self as _);
            self.key = Some(key);

            let scope = Scope::default();
            scope.inner.borrow_mut().contexts = cx_ref.contexts.clone();
            self.scope = Some(scope);

            cx_ref.node = Some(key);
            cx_ref.scope = Some(self.scope.clone().unwrap());
            drop(cx_ref);

            trace!("building {:?}: {}", key.data(), self.name());

            let view = unsafe { mem::transmute(&self.view) };
            let body = (self.builder)(view);

            let parent_contexts = {
                let mut cx_ref = cx.inner.borrow_mut();
                let mut scope = self.scope.as_ref().unwrap().inner.borrow_mut();
                scope.hook_idx = 0;
                mem::replace(&mut cx_ref.contexts, scope.contexts.clone())
            };

            self.body = Some(body);
            self.body.as_mut().unwrap().build();

            let mut cx_ref = cx.inner.borrow_mut();
            cx_ref.contexts = parent_contexts;
        }
    }

    unsafe fn rebuild(&mut self, last: &mut dyn Any, is_changed: bool) {
        let last = (*last).downcast_mut::<Self>().unwrap();
        let cx = Runtime::current();
        let mut cx_ref = cx.inner.borrow_mut();

        let key = last.key.unwrap();
        self.key = Some(key);
        self.scope = last.scope.clone();

        if is_changed {
            let mut scope = self.scope.as_ref().unwrap().inner.borrow_mut();
            for (name, value) in cx_ref.contexts.iter() {
                if !scope.contexts.contains_key(name) {
                    scope.contexts.insert(*name, value.clone());
                }
            }
            drop(scope);

            cx_ref.node = Some(key);
            cx_ref.scope = Some(self.scope.clone().unwrap());
            drop(cx_ref);

            trace!("rebuilding {:?}: {}", key.data(), self.name());

            let view = unsafe { mem::transmute(&self.view) };
            let body = (self.builder)(view);

            let parent_contexts = {
                let mut cx_ref = cx.inner.borrow_mut();
                let mut scope = self.scope.as_ref().unwrap().inner.borrow_mut();
                scope.hook_idx = 0;
                mem::replace(&mut cx_ref.contexts, scope.contexts.clone())
            };

            self.body = Some(body);

            self.body
                .as_mut()
                .unwrap()
                .rebuild(last.body.as_mut().unwrap(), is_changed);

            let mut cx_ref = cx.inner.borrow_mut();
            cx_ref.contexts = parent_contexts;
        } else {
            trace!("skipping {:?}: {}", self.key.unwrap().data(), self.name());

            self.body = last.body.take();
        }
    }

    unsafe fn remove(&mut self) {
        let cx = Runtime::current();
        let mut cx_ref = cx.inner.borrow_mut();
        let key = self.key.unwrap();
        cx_ref.nodes.remove(key);
        drop(cx_ref);

        trace!("removing {:?}: {}", key.data(), self.name());

        for dropper in &mut self.scope.as_ref().unwrap().inner.borrow_mut().droppers {
            dropper()
        }

        self.body.as_mut().unwrap().remove();
    }
}
