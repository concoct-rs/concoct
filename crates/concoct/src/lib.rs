use slotmap::{DefaultKey, SlotMap};
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::task::{Poll, Waker};
use std::{cell::RefCell, mem, rc::Rc};

pub mod body;
pub use self::body::Body;
use body::Empty;

pub mod hook;

pub mod view;
pub use self::view::View;

#[derive(Default)]
struct ScopeInner {
    contexts: HashMap<TypeId, Rc<dyn Any>>,
    hooks: Vec<Rc<dyn Any>>,
    hook_idx: usize,
    droppers: Vec<Box<dyn FnMut()>>,
}

#[derive(Clone, Default)]
pub struct Scope {
    inner: Rc<RefCell<ScopeInner>>,
}

#[derive(Default)]
struct ContextInner {
    node: Option<DefaultKey>,
    pending: VecDeque<DefaultKey>,
    scope: Option<Scope>,
    nodes: SlotMap<DefaultKey, *mut dyn Tree>,
    waker: Option<Waker>,
    contexts: HashMap<TypeId, Rc<dyn Any>>,
}

#[derive(Clone, Default)]
pub struct Context {
    inner: Rc<RefCell<ContextInner>>,
}

impl Context {
    pub fn enter(&self) {
        CONTEXT
            .try_with(|cell| *cell.borrow_mut() = Some(self.clone()))
            .unwrap();
    }

    pub fn current() -> Self {
        CONTEXT
            .try_with(|cell| cell.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub async fn rebuild(&self) {
        futures::future::poll_fn(|cx| {
            let mut inner = self.inner.borrow_mut();
            inner.waker = Some(cx.waker().clone());

            if let Some(key) = inner.pending.pop_front() {
                let raw = inner.nodes[key];
                drop(inner);

                let pending = unsafe { &mut *raw };
                pending.build();
            }

            Poll::Pending
        })
        .await
    }
}

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

pub struct Node<V, B, F> {
    view: V,
    body: Option<B>,
    builder: F,
    scope: Option<Scope>,
    key: Option<DefaultKey>,
}

pub trait Tree: 'static {
    fn build(&mut self);

    fn rebuild(&mut self, last: &mut dyn Any);

    fn remove(&mut self);
}

impl<T: Tree> Tree for Option<T> {
    fn build(&mut self) {
        if let Some(tree) = self {
            tree.build()
        }
    }

    fn rebuild(&mut self, last: &mut dyn Any) {
        if let Some(tree) = self {
            if let Some(last_tree) = last.downcast_mut::<Self>().unwrap() {
                tree.rebuild(last_tree)
            } else {
                tree.build();
            }
        } else if let Some(last_tree) = last.downcast_mut::<Self>().unwrap() {
            last_tree.remove();
        }
    }

    fn remove(&mut self) {
        if let Some(tree) = self {
            tree.remove()
        }
    }
}

macro_rules! one_of {
    ($name:tt, $($t:tt),*) => {
        pub enum $name<$($t),*> {
            $($t($t)),*
        }

        impl<$($t: Body),*> Body for $name<$($t),*> {
            fn into_tree(self) -> impl Tree {
                match self {
                    $(
                        $name::$t(body) => $name::$t(body.into_tree()),
                    )*
                }
            }
        }

        impl<$($t: Tree),*> Tree for $name<$($t),*> {
            fn build(&mut self) {
                match self {
                    $(
                        $name::$t(tree) => tree.build(),
                    )*
                }
            }

            fn rebuild(&mut self, last: &mut dyn Any) {
                let last =  last.downcast_mut::<Self>().unwrap();
                match (self, last) {
                    $(
                        ($name::$t(tree), $name::$t(last_tree)) => {
                            tree.rebuild(last_tree)
                        }
                    ),*
                    (me, last) => {
                        last.remove();
                        me.build();
                    }
                }

            }

            fn remove(&mut self) {
                match self {
                    $(
                        $name::$t(tree) => tree.remove(),
                    )*
                }
            }
        }
    };
}

one_of!(OneOf2, A, B);
one_of!(OneOf3, A, B, C);
one_of!(OneOf4, A, B, C, D);
one_of!(OneOf5, A, B, C, D, E);
one_of!(OneOf6, A, B, C, D, E, F);
one_of!(OneOf7, A, B, C, D, E, F, G);
one_of!(OneOf8, A, B, C, D, E, F, G, H);

macro_rules! impl_tree_for_tuple {
    ($($t:tt : $idx:tt),*) => {
        impl<$($t: Tree),*> Tree for ($($t),*) {
            fn build(&mut self) {
               $(
                    self.$idx.build();
               )*
            }

            fn rebuild(&mut self, last: &mut dyn Any) {
                if let Some(last) = last.downcast_mut::<Self>() {
                    $(
                        self.$idx.rebuild(&mut last.$idx);
                    )*
                }
            }

            fn remove(&mut self) {
                $(
                     self.$idx.remove();
                )*
             }
        }
    };
}

impl_tree_for_tuple!(V1: 0, V2: 1);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8, V10: 9);

impl Tree for Empty {
    fn build(&mut self) {}

    fn rebuild(&mut self, _last: &mut dyn Any) {}

    fn remove(&mut self) {}
}

impl<K: Hash + Eq + 'static, T: Tree> Tree for Vec<(K, T)> {
    fn build(&mut self) {
        for (_, body) in self.iter_mut() {
            body.build()
        }
    }

    fn rebuild(&mut self, last: &mut dyn Any) {
        let mut visited = HashSet::new();
        let last = last.downcast_mut::<Self>().unwrap();

        for (key, body) in self.iter_mut() {
            if let Some((_, last_body)) = last.iter_mut().find(|(last_key, _)| last_key == key) {
                body.rebuild(last_body);
                visited.insert(key);
            } else {
                body.build();
            }
        }

        for (key, body) in last.iter_mut() {
            if !visited.contains(key) {
                body.remove();
            }
        }
    }

    fn remove(&mut self) {
        for (_, body) in self.iter_mut() {
            body.remove()
        }
    }
}

impl<V, B, F> Tree for Node<V, B, F>
where
    V: View,
    B: Tree + 'static,
    F: FnMut(&'static V) -> B + 'static,
{
    fn build(&mut self) {
        let cx = Context::current();
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

            let view = unsafe { mem::transmute(&self.view) };
            let body = (self.builder)(view);

            let parent_contexts = {
                let mut cx_ref = cx.inner.borrow_mut();
                let mut scope = self.scope.as_ref().unwrap().inner.borrow_mut();
                scope.hook_idx = 0;
                mem::replace(&mut cx_ref.contexts, scope.contexts.clone())
            };

            let mut last_body = mem::replace(&mut self.body, Some(body)).unwrap();
            self.body.as_mut().unwrap().rebuild(&mut last_body);

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

    fn rebuild(&mut self, last: &mut dyn Any) {
        let last = (*last).downcast_mut::<Self>().unwrap();
        let cx = Context::current();
        let mut cx_ref = cx.inner.borrow_mut();

        let key = last.key.unwrap();
        self.key = Some(key);
        self.scope = last.scope.clone();

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
            .rebuild(last.body.as_mut().unwrap());

        let mut cx_ref = cx.inner.borrow_mut();
        cx_ref.contexts = parent_contexts;
    }

    fn remove(&mut self) {
        let cx = Context::current();
        let mut cx_ref = cx.inner.borrow_mut();
        let key = self.key.unwrap();
        cx_ref.nodes.remove(key);
        drop(cx_ref);

        for dropper in &mut self.scope.as_ref().unwrap().inner.borrow_mut().droppers {
            dropper()
        }

        self.body.as_mut().unwrap().remove();
    }
}

pub async fn run(view: impl View) {
    let cx = Context::default();
    cx.enter();

    let mut tree = view.into_tree();
    tree.build();

    loop {
        cx.rebuild().await
    }
}

pub struct TextViewContext {
    view: RefCell<Box<dyn FnMut(Cow<'static, str>)>>,
}

impl TextViewContext {
    pub fn new(view: impl FnMut(Cow<'static, str>) + 'static) -> Self {
        Self {
            view: RefCell::new(Box::new(view)),
        }
    }
}
