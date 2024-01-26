//! Concoct is a framework for user-interfaces in Rust.
//!
//! This crate provides a virtual DOM and state management system for any backend.
//! Concoct uses static typing to describe your UI at compile-time to create an efficient
//! tree without allocations.
//!
//! ```ignore
//! #[derive(Default)]
//! struct Counter {
//!     count: i32,
//! }
//!
//! impl View<Counter> for Counter {
//!     fn body(&mut self, _cx: &Scope<Counter>) -> impl View<Counter> {
//!         (
//!             format!("High five count: {}", self.count),
//!             html::button("Up high!").on_click(|state: &mut Self, _event| state.count += 1),
//!             html::button("Down low!").on_click(|state: &mut Self, _event| state.count -= 1),
//!         )
//!     }
//! }
//! ```

#![deny(missing_docs)]

use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell, UnsafeCell},
    ops::DerefMut,
    rc::Rc,
    task::Waker,
};

mod action;
pub use self::action::{Action, IntoAction};

pub mod hook;

mod vdom;
pub use self::vdom::VirtualDom;

pub mod view;
pub use self::view::View;

/// Handle to update a scope.
pub struct Handle<T, A = ()> {
    update: Rc<dyn Fn(Rc<dyn Fn(&mut T) -> Option<A>>)>,
}

impl<T, A> Handle<T, A> {
    /// Send an update to the virtual dom from this handle's scope.
    pub fn update(&self, f: Rc<dyn Fn(&mut T) -> Option<A>>) {
        (self.update)(f)
    }
}

/// Scope of a view.
pub struct Scope<T, A = ()> {
    key: DefaultKey,
    parent: Option<DefaultKey>,
    node: Node,
    update: Rc<dyn Fn(Rc<dyn Fn(&mut T) -> Option<A>>)>,
    is_empty: Cell<bool>,
    nodes: Rc<RefCell<SlotMap<DefaultKey, Node>>>,
    contexts: RefCell<FxHashMap<TypeId, Rc<dyn Any>>>,
}

impl<T, A> Scope<T, A> {
    /// Create a handle to this scope.
    pub fn handle(&self) -> Handle<T, A> {
        Handle {
            update: self.update.clone(),
        }
    }
}

#[derive(Default)]
struct NodeInner {
    hooks: Vec<UnsafeCell<Box<dyn Any>>>,
    hook_idx: usize,
    children: Vec<DefaultKey>,
}

#[derive(Clone, Default)]
struct Node {
    inner: Rc<RefCell<NodeInner>>,
}

struct Channel<T> {
    updates: Vec<Rc<dyn Fn(&mut T) -> Option<()>>>,
    waker: Option<Waker>,
}

fn build_inner<T, A>(view: &mut impl View<T, A>, cx: &Scope<T, A>) {
    let node = Node::default();
    let key = cx.nodes.borrow_mut().insert(node.clone());
    cx.node.inner.borrow_mut().children.push(key);

    let child_cx = Scope {
        key,
        parent: Some(cx.key),
        node,
        update: cx.update.clone(),
        is_empty: Cell::new(false),
        nodes: cx.nodes.clone(),
        contexts: cx.contexts.clone(),
    };

    let mut body = view.body(&child_cx);
    if !child_cx.is_empty.get() {
        build_inner(&mut body, &child_cx);
    }
}

fn rebuild_inner<T, A>(view: &mut impl View<T, A>, cx: &Scope<T, A>) {
    for child_key in &cx.node.inner.borrow().children {
        let node = cx.nodes.borrow().get(*child_key).cloned();
        if let Some(node) = node {
            node.inner.borrow_mut().hook_idx = 0;

            let child_cx = Scope {
                key: *child_key,
                node,
                parent: Some(cx.key),
                update: cx.update.clone(),
                is_empty: Cell::new(false),
                nodes: cx.nodes.clone(),
                contexts: cx.contexts.clone(),
            };

            let mut body = view.body(&child_cx);
            if !child_cx.is_empty.get() {
                rebuild_inner(&mut body, &child_cx);
            }
        }
    }
}

/// Run a view on a new virtual dom.
pub async fn run<T, V>(content: V)
where
    T: 'static,
    V: View<T> + DerefMut<Target = T>,
{
    let mut vdom = VirtualDom::new(content);
    vdom.build();
    loop {
        vdom.rebuild().await;
    }
}
