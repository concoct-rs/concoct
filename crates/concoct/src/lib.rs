use hook::use_context;
use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell, UnsafeCell},
    rc::Rc,
};

pub mod hook;

pub enum ActionResult<A> {
    Action(A),
    Rebuild,
}

pub struct Scope<T, A = ()> {
    pub key: DefaultKey,
    node: Node,
    update: Rc<dyn Fn(Rc<dyn Fn(T) -> Option<ActionResult<A>>>)>,
    is_empty: Cell<bool>,
    nodes: Rc<RefCell<SlotMap<DefaultKey, Node>>>,
    contexts: RefCell<FxHashMap<TypeId, Rc<dyn Any>>>,
}

pub trait View<T, A = ()> {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A>;
}

impl<T, A> View<T, A> for () {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        cx.is_empty.set(true);
    }
}

impl<T, A, V: View<T, A>> View<T, A> for &mut V {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        (&mut **self).body(cx)
    }
}

macro_rules! impl_view_for_tuple {
    ($($t:tt : $idx:tt),*) => {
        impl<T, A, $($t: View<T, A>),*> View<T, A> for ($($t),*) {
            fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
                if cx.node.inner.borrow().children.is_empty() {
                    $( build_inner(&mut self.$idx, cx); )*
                } else {
                    $( {
                        let key = cx.node.inner.borrow().children[$idx];
                        let node = cx.nodes.borrow()[key].clone();

                        let cx = Scope {
                            key,
                            node,
                            update: cx.update.clone(),
                            is_empty: Cell::new(false),
                            nodes: cx.nodes.clone(),
                            contexts: cx.contexts.clone()
                        };

                        let mut body = self.$idx.body(&cx);
                        if !cx.is_empty.get() {
                            rebuild_inner(&mut body, &cx);
                        }
                    } )*
                }
                cx.is_empty.set(true);
            }
        }
    };
}

impl_view_for_tuple!(V1: 0, V2: 1);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8, V10: 9);

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

pub struct VirtualDom<V> {
    content: V,
    nodes: Rc<RefCell<SlotMap<DefaultKey, Node>>>,
    pending_updates: Vec<Box<dyn FnMut()>>,
    root_key: Option<DefaultKey>,
}

impl<V> VirtualDom<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            nodes: Rc::default(),
            pending_updates: Vec::new(),
            root_key: None,
        }
    }

    pub fn build(&mut self)
    where
        V: View<V>,
    {
        let node = Node::default();
        let root_key = self.nodes.borrow_mut().insert(node.clone());
        self.root_key = Some(root_key);

        let cx = Scope {
            key: root_key,
            node,
            update: Rc::new(|_f| {}),
            is_empty: Cell::new(false),
            nodes: self.nodes.clone(),
            contexts: Default::default(),
        };
        build_inner(&mut self.content, &cx)
    }

    pub fn rebuild(&mut self)
    where
        V: View<V>,
    {
        let root_key = self.root_key.unwrap();
        let node = self.nodes.borrow()[root_key].clone();
        let cx = Scope {
            key: root_key,
            node,
            update: Rc::new(|_| {}),
            is_empty: Cell::new(false),
            nodes: self.nodes.clone(),
            contexts: Default::default(),
        };
        rebuild_inner(&mut self.content, &cx)
    }
}

fn build_inner<T, A>(view: &mut impl View<T, A>, cx: &Scope<T, A>) {
    let node = Node::default();
    let key = cx.nodes.borrow_mut().insert(node.clone());
    cx.node.inner.borrow_mut().children.push(key);

    let child_cx = Scope {
        key,
        node,
        update: Rc::new(|_f| {}),
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

impl<T: 'static, A: 'static> View<T, A> for &str {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        let text_cx: Rc<TextViewContext<T, A>> = use_context(cx);
        let mut view = text_cx.view.borrow_mut();
        view(cx, self)
    }
}

impl<T: 'static, A: 'static> View<T, A> for String {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        let text_cx: Rc<TextViewContext<T, A>> = use_context(cx);
        let mut view = text_cx.view.borrow_mut();
        view(cx, self)
    }
}

pub fn run<V: View<V>>(content: V) {
    let mut vdom = VirtualDom::new(content);
    vdom.build();
    vdom.rebuild();
}
