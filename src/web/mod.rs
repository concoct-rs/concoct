use std::{cell::RefCell, rc::Rc};
use web_sys::{Document, HtmlElement};

use crate::{
    html::{Builder, Html, Platform},
    Composition, IntoComposable,
};

thread_local! {
    static HTML_CONTEXT: RefCell<Option<WebContext>> = RefCell::default();
}

struct Inner {
    document: Document,
    body: HtmlElement,
}

#[derive(Clone)]
pub struct WebContext {
    inner: Rc<RefCell<Inner>>,
}

impl WebContext {
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

pub fn html() -> Html<WebHtml> {
    Html::new(WebHtml {})
}

#[derive(PartialEq, Eq)]
pub struct WebHtml {}

impl Platform for WebHtml {
    fn html(&mut self, html: &mut Builder) -> impl IntoComposable {
        let cx = WebContext::current();
        let inner = cx.inner.borrow_mut();
        let element = inner.document.create_element("div").unwrap();

        for (name, value) in &html.attrs {
            element.set_attribute(&name, &value).unwrap();
        }

        inner.body.append_child(&element).unwrap();
    }
}

pub fn run<C>(content: fn() -> C)
where
    C: IntoComposable + 'static,
{
    let cx = WebContext::new();
    cx.enter();

    let mut composition = Composition::new(content);
    composition.build()
}
