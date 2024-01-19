use std::{cell::RefCell, mem, rc::Rc};

mod body;
use body::Empty;

pub use self::body::Body;

mod view;
pub use self::view::View;

#[derive(Default)]
struct ContextInner {
    node: Option<*mut dyn Tree>,
    pending: Vec<*mut dyn Tree>,
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
        Context::current().inner.borrow_mut().node = Some(self as _);

        let view = unsafe { mem::transmute(&self.view) };
        let body = (self.builder)(view);
        self.body = Some(body);

        self.body.as_mut().unwrap().build();
    }
}

impl<V: View> Body for V {
    fn tree(self) -> impl Tree {
        Node {
            view: self,
            body: None,
            builder: |me: &'static V| me.body().tree(),
        }
    }
}
