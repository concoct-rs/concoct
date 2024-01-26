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

use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell, UnsafeCell},
    ops::DerefMut,
    rc::Rc,
    task::Waker,
};

pub mod hook;

pub mod view;
pub use self::view::View;

pub enum ActionResult<A> {
    Action(A),
    Rebuild,
}

pub struct Handle<T, A = ()> {
    update: Rc<dyn Fn(Rc<dyn Fn(&mut T) -> Option<ActionResult<A>>>)>,
}

impl<T, A> Handle<T, A> {
    pub fn update(&self, f: Rc<dyn Fn(&mut T) -> Option<ActionResult<A>>>) {
        (self.update)(f)
    }
}

pub struct Scope<T, A = ()> {
    key: DefaultKey,
    node: Node,
    update: Rc<dyn Fn(Rc<dyn Fn(&mut T) -> Option<ActionResult<A>>>)>,
    is_empty: Cell<bool>,
    nodes: Rc<RefCell<SlotMap<DefaultKey, Node>>>,
    contexts: RefCell<FxHashMap<TypeId, Rc<dyn Any>>>,
}

impl<T, A> Scope<T, A> {
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
    updates: Vec<Rc<dyn Fn(&mut T) -> Option<ActionResult<()>>>>,
    waker: Option<Waker>,
}

pub struct VirtualDom<T, V> {
    content: V,
    nodes: Rc<RefCell<SlotMap<DefaultKey, Node>>>,
    channel: Rc<RefCell<Channel<T>>>,
    root_key: Option<DefaultKey>,
}

impl<T, V> VirtualDom<T, V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            nodes: Rc::default(),
            channel: Rc::new(RefCell::new(Channel {
                updates: Vec::new(),
                waker: None,
            })),
            root_key: None,
        }
    }

    pub fn build(&mut self)
    where
        T: 'static,
        V: View<T> + DerefMut<Target = T>,
    {
        let node = Node::default();
        let root_key = self.nodes.borrow_mut().insert(node.clone());
        self.root_key = Some(root_key);

        let channel = self.channel.clone();
        let cx = Scope {
            key: root_key,
            node,
            update: Rc::new(move |f| {
                let mut channel_ref = channel.borrow_mut();
                channel_ref.updates.push(f);
                if let Some(waker) = channel_ref.waker.take() {
                    waker.wake();
                }
            }),
            is_empty: Cell::new(false),
            nodes: self.nodes.clone(),
            contexts: Default::default(),
        };
        build_inner(&mut self.content, &cx)
    }

    pub async fn rebuild(&mut self)
    where
        T: 'static,
        V: View<T> + DerefMut<Target = T>,
    {
        futures::future::poll_fn(|cx| {
            self.channel.borrow_mut().waker = Some(cx.waker().clone());

            let mut is_updated = false;
            loop {
                let mut channel_ref = self.channel.borrow_mut();
                if let Some(update) = channel_ref.updates.pop() {
                    update(&mut self.content);
                    is_updated = true;
                } else {
                    break;
                }
            }

            if is_updated {
                let root_key = self.root_key.unwrap();
                let node = self.nodes.borrow()[root_key].clone();

                let channel = self.channel.clone();
                let cx = Scope {
                    key: root_key,
                    node,
                    update: Rc::new(move |f| {
                        let mut channel_ref = channel.borrow_mut();
                        channel_ref.updates.push(f);
                        if let Some(waker) = channel_ref.waker.take() {
                            waker.wake();
                        }
                    }),
                    is_empty: Cell::new(false),
                    nodes: self.nodes.clone(),
                    contexts: Default::default(),
                };
                rebuild_inner(&mut self.content, &cx);
            }

            std::task::Poll::Pending
        })
        .await
    }
}

fn build_inner<T, A>(view: &mut impl View<T, A>, cx: &Scope<T, A>) {
    let node = Node::default();
    let key = cx.nodes.borrow_mut().insert(node.clone());
    cx.node.inner.borrow_mut().children.push(key);

    let child_cx = Scope {
        key,
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
        let node = cx.nodes.borrow()[*child_key].clone();
        node.inner.borrow_mut().hook_idx = 0;

        let child_cx = Scope {
            key: *child_key,
            node,
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

/// Provider for a platform-specific text view.
///
/// If you're writing a custom backend, you can use this to override
/// the default implementation of `View` for string types (like `&str` and `String`).
///
/// To expose it to child views, use [`use_provider`](`crate::hook::use_provider`).
pub struct TextViewContext<T, A> {
    view: RefCell<Box<dyn FnMut(&Scope<T, A>, &str)>>,
}

impl<T, A> TextViewContext<T, A> {
    /// Create a text view context from a view function.
    ///
    /// Text-based views, such as `&str` or `String` will call
    /// this view function on when rendered.
    pub fn new(view: impl FnMut(&Scope<T, A>, &str) + 'static) -> Self {
        Self {
            view: RefCell::new(Box::new(view)),
        }
    }
}

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

pub trait Action {}

pub trait IntoAction<A>: sealed::Sealed {
    fn into_action(self) -> Option<ActionResult<A>>;
}
mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for () {}

impl<A> IntoAction<A> for () {
    fn into_action(self) -> Option<ActionResult<A>> {
        None
    }
}

impl<A: Action> sealed::Sealed for A {}

impl<A: Action> IntoAction<A> for A {
    fn into_action(self) -> Option<ActionResult<A>> {
        Some(ActionResult::Action(self))
    }
}

impl<A: Action> sealed::Sealed for Option<ActionResult<A>> {}

impl<A: Action> IntoAction<A> for Option<ActionResult<A>> {
    fn into_action(self) -> Option<ActionResult<A>> {
        self
    }
}
