use std::{cell::RefCell, rc::Rc};
use web_sys::{Document, HtmlElement};

use crate::{Composable, Composition, IntoComposable};

thread_local! {
    static HTML_CONTEXT: RefCell<Option<HtmlContext>> = RefCell::default();
}

struct Inner {
    document: Document,
    body: HtmlElement,
}

#[derive(Clone)]
pub struct HtmlContext {
    inner: Rc<RefCell<Inner>>,
}

impl HtmlContext {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        Self {
            inner: Rc::new(RefCell::new(Inner { document, body })),
        }
    }

    pub fn current() -> Self {
        HTML_CONTEXT
            .try_with(|cx| cx.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub fn enter(self) {
        HTML_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self))
            .unwrap()
    }
}

#[derive(PartialEq, Eq)]
pub struct Html {}

impl Composable for Html {
    fn compose(&mut self) -> impl IntoComposable {
        let cx = HtmlContext::current();
        let inner = cx.inner.borrow_mut();
        let element = inner.document.create_element("div").unwrap();
        inner.body.append_child(&element).unwrap();
    }
}

pub fn run<C>(content: fn() -> C)
where
    C: IntoComposable + 'static,
{
    let cx = HtmlContext::new();
    cx.enter();

    let mut composition = Composition::new(content);
    composition.build()
}
