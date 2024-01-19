use std::any::Any;
use std::{cell::RefCell, mem, rc::Rc};

pub mod body;
pub use self::body::Body;
use body::Empty;

pub mod view;
pub use self::view::View;

pub fn use_ref<T: 'static>(make_value: impl FnOnce() -> T) -> Rc<T> {
    let cx = Context::current();
    let cx_ref = cx.inner.borrow();
    let scope = &mut *cx_ref.scope.as_ref().unwrap().inner.borrow_mut();

    if let Some(any) = scope.hooks.get(scope.hook_idx) {
        scope.hook_idx += 1;
        Rc::downcast(any.clone()).unwrap()
    } else {
        let value = Rc::new(make_value());
        scope.hooks.push(value.clone());
        value
    }
}

#[derive(Default)]
struct ScopeInner {
    hooks: Vec<Rc<dyn Any>>,
    hook_idx: usize,
}

#[derive(Clone, Default)]
pub struct Scope {
    inner: Rc<RefCell<ScopeInner>>,
}

#[derive(Default)]
struct ContextInner {
    node: Option<*mut dyn Tree>,
    pending: Vec<*mut dyn Tree>,
    scope: Option<Scope>,
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

    pub fn rebuild(&self) {
        let raw = self.inner.borrow_mut().pending.pop().unwrap();
        let pending = unsafe { &mut *raw };
        pending.build();
    }
}

pub fn request_update() {
    let cx = Context::current();
    let cx = &mut *cx.inner.borrow_mut();
    cx.pending.push(cx.node.unwrap() as *mut _);
}

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

pub struct Node<V, B, F> {
    view: V,
    body: Option<B>,
    builder: F,
    scope: Scope,
}

pub trait Tree {
    fn build(&mut self);
}

impl Tree for Empty {
    fn build(&mut self) {}
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

        cx_ref.node = Some(self as _);
        cx_ref.scope = Some(self.scope.clone());
        drop(cx_ref);

        let view = unsafe { mem::transmute(&self.view) };
        let body = (self.builder)(view);
        self.body = Some(body);
        self.body.as_mut().unwrap().build();

        let _cx_ref = cx.inner.borrow_mut();
    }
}
