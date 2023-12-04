use crate::{
    html::{AttributeValue, Builder, Html, HtmlParent, HtmlPlatform},
    use_context, use_provider, use_ref, Child, IntoView, LocalContext, Platform, Tree,
    TASK_CONTEXT, AnyChild,
};
use std::{borrow::Cow, cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
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

macro_rules! impl_html_tags {
    ($($tag:ident),*) => {
        $(
            pub fn $tag() -> Html<WebHtml> {
                html(stringify!($tag))
            }
        )*
    };
}

impl_html_tags!(
    a, abbr, address, article, aside, audio, b, bdi, bdo, blockquote, body, br, button, canvas,
    caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn, dialog, div, dl, dt,
    em, embed, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4, h5, h6, head, header,
    hgroup, hr, i, iframe, img, input, ins, kbd, label, legend, li, link, main, map, mark, meta,
    meter, nav, noscript, object, ol, optgroup, option, output, p, param, picture, pre, progress,
    q, rp, rt, ruby, s, samp, script, section, select, small, source, span, strong, style, sub,
    summary, sup, table, tbody, td, template, textarea, tfoot, th, thead, time, title, tr, track,
    u, ul, var, video, wbr
);

pub fn html(tag: impl Into<Cow<'static, str>>) -> Html<WebHtml> {
    Html::new(tag, WebHtml {})
}

#[derive(PartialEq, Eq)]
pub struct WebHtml {}

impl HtmlPlatform for WebHtml {
    fn html(
        &mut self,
        html: &mut Builder,
        parent: Option<Node>,
        child: Option<AnyChild>,
    ) -> impl IntoView {
        let callbacks = use_ref(|| Vec::new());
        let element = use_ref(|| {
            let cx = WebContext::current();
            let inner = cx.inner.borrow_mut();
            let element = inner.document.create_element(&html.tag).unwrap();

            for (name, value) in &html.attrs {
                match &value {
                    AttributeValue::String(s) => element.set_attribute(&name, s).unwrap(),
                    AttributeValue::Callback(callback) => {
                        let callback = callback.clone();
                        let local_cx = LocalContext::current();
                        let task_cx = TASK_CONTEXT
                            .try_with(|cx| cx.clone().borrow().as_ref().unwrap().clone())
                            .unwrap();
                        let listener: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
                            local_cx.clone().enter();
                            TASK_CONTEXT
                                .try_with(|cx| *cx.borrow_mut() = Some(task_cx.clone()))
                                .unwrap();
                            callback.borrow_mut()();
                        }));
                        element
                            .add_event_listener_with_callback(
                                &name,
                                listener.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                        callbacks.get_mut().push(listener);
                    }
                }
            }

            let parent = parent.as_ref().unwrap_or(inner.body.unchecked_ref());
            parent.append_child(&element).unwrap();

            element
        });

        use_provider(|| HtmlParent {
            node: element.get().clone().unchecked_into(),
        });

        child
    }
}

pub struct Web;

impl Platform for Web {
    fn text(&self, s: &str) -> Box<dyn crate::AnyView> {
        let parent = use_context::<HtmlParent>().map(|parent| parent.get().node.clone());

        let state = use_ref(|| s.to_string());

        let node = use_ref(|| {
            let cx = WebContext::current();
            let inner = cx.inner.borrow_mut();
            let node = inner.document.create_text_node(s);

            let parent = parent.as_ref().unwrap_or(inner.body.unchecked_ref());
            parent.append_child(&node).unwrap();
            node
        });

        if s != &*state.get() {
            node.get_mut().set_text_content(Some(s));
            *state.get_mut() = s.to_string();
        }

        Box::new(())
    }
}

pub fn run(content: impl IntoView) {
    let cx = WebContext::new();
    cx.enter();

    let mut composition = Tree::new(Web, content);
    composition.build();

    spawn_local(async move {
        loop {
            composition.rebuild().await;
        }
    })
}
