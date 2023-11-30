use crate::{
    html::{Builder, Html, HtmlParent, HtmlPlatform},
    use_provider, Child, IntoView, Platform, Tree,
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, Node};

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

pub fn div<C>(child: C) -> Html<WebHtml, C> {
    Html::new(WebHtml {}, child)
}

#[derive(PartialEq, Eq)]
pub struct WebHtml {}

impl HtmlPlatform for WebHtml {
    fn html<C: IntoView>(
        &mut self,
        html: &mut Builder,
        parent: Option<Node>,
        child: Child<C>,
    ) -> impl IntoView {
        let cx = WebContext::current();
        let inner = cx.inner.borrow_mut();
        let element = inner.document.create_element("div").unwrap();

        for (name, value) in &html.attrs {
            element.set_attribute(&name, &value).unwrap();
        }

        let parent = parent.as_ref().unwrap_or(inner.body.unchecked_ref());
        parent.append_child(&element).unwrap();

        use_provider(|| HtmlParent {
            node: element.unchecked_into(),
        });
        child
    }
}

pub struct Web;

impl Platform for Web {
    fn from_str(&mut self, s: &str) -> Box<dyn crate::AnyView> {
        let cx = WebContext::current();
        let inner = cx.inner.borrow_mut();
        let node = inner.document.create_text_node(s);

        inner.body.append_child(&node).unwrap();

        Box::new(())
    }
}

pub fn run(content: impl IntoView) {
    let cx = WebContext::new();
    cx.enter();

    let mut composition = Tree::new(Web, content);
    composition.build()
}
