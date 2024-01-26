use crate::{vdom::Node, Handle, View};
use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    mem,
    rc::Rc,
};

/// Scope of a view.
pub struct Scope<T, A = ()> {
    pub(crate) key: DefaultKey,
    pub(crate) parent: Option<DefaultKey>,
    pub(crate) node: Node,
    pub(crate) update: Rc<dyn Fn(Rc<dyn Fn(&Handle<T, A>, &mut T) -> Option<A>>)>,
    pub(crate) is_empty: Cell<bool>,
    pub(crate) nodes: Rc<RefCell<SlotMap<DefaultKey, Node>>>,
    pub(crate) contexts: RefCell<FxHashMap<TypeId, Rc<dyn Any>>>,
}

impl<T, A> Scope<T, A> {
    /// Create a handle to this scope.
    pub fn handle(&self) -> Handle<T, A> {
        Handle {
            update: self.update.clone(),
        }
    }

    /// Manually build a tree of views.
    pub fn build(&self, mut view: impl View<T, A>) {
        let node = Node::default();
        let key = self.nodes.borrow_mut().insert(node.clone());
        self.node.inner.borrow_mut().children.push(key);

        let child_cx = Scope {
            key,
            parent: Some(self.key),
            node,
            update: self.update.clone(),
            is_empty: Cell::new(false),
            nodes: self.nodes.clone(),
            contexts: self.contexts.clone(),
        };

        let body = view.body(&child_cx);
        if !child_cx.is_empty.get() {
            child_cx.build(body);
        }
    }

    /// Manually rebuild a tree of views.
    ///
    /// By calling this function you must make sure all references to hooks are dropped.
    pub unsafe fn rebuild(&self, mut view: impl View<T, A>) {
        for child_key in &self.node.inner.borrow().children {
            let node = self.nodes.borrow().get(*child_key).cloned();
            if let Some(node) = node {
                node.inner.borrow_mut().hook_idx = 0;

                let child_cx = Scope {
                    key: *child_key,
                    node,
                    parent: Some(self.key),
                    update: self.update.clone(),
                    is_empty: Cell::new(false),
                    nodes: self.nodes.clone(),
                    contexts: self.contexts.clone(),
                };

                let body = view.body(&child_cx);
                if !child_cx.is_empty.get() {
                    child_cx.rebuild(body);
                }
            }
        }
    }

    /// Remove all of this scope's children from the virtual dom.
    pub fn clear(&self) {
        let mut nodes_ref = self.nodes.borrow_mut();
        let mut stack = Vec::new();
        for child_key in &mem::take(&mut self.node.inner.borrow_mut().children) {
            let child_node = nodes_ref[*child_key].clone();
            stack.push((*child_key, child_node));
        }

        while let Some((key, node)) = stack.pop() {
            nodes_ref.remove(key);
            for child_key in &node.inner.borrow().children {
                let child_node = nodes_ref[*child_key].clone();
                stack.push((*child_key, child_node));
            }
        }
    }
}
