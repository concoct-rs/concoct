use crate::{
    hook::{use_context, use_provider, use_ref},
    Body, TextViewContext, View,
};
use std::{cell::RefCell, rc::Rc};
use web_sys::{wasm_bindgen::closure::Closure, Document, Element, Event, Node, Text, Window};

pub mod html;

struct WebContext {
    window: Window,
    document: Document,
    parent: Node,
}

pub struct WebRoot<B> {
    pub body: Rc<B>,
}

impl<B: View> View for WebRoot<B> {
    fn body(&self) -> impl Body {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        use_provider(WebContext {
            window,
            document,
            parent: body.into(),
        });

        use_provider(TextViewContext::new(|s| {
            let web_cx = use_context::<WebContext>().unwrap();

            let data = use_ref(|| RefCell::new((s.clone(), None::<Text>)));
            let (last, node_cell) = &mut *data.borrow_mut();

            if let Some(node) = node_cell {
                if s != *last {
                    node.set_text_content(Some(&s));
                    *last = s.clone();
                }
            } else {
                let node = web_cx.document.create_text_node(&s);
                web_cx.parent.append_child(&node).unwrap();
                *node_cell = Some(node);
            }
        }));

        self.body.clone()
    }
}

#[derive(Default)]
struct Data {
    element: Option<Element>,
    callbacks: Vec<(
        Closure<dyn FnMut(Event)>,
        Rc<RefCell<Rc<RefCell<dyn FnMut(Event)>>>>,
    )>,
}

macro_rules! impl_string_view {
    ($t:ty) => {
        impl View for $t {
            fn body(&self) -> impl Body {
                let cx = use_context::<TextViewContext>().unwrap();
                let mut view = cx.view.borrow_mut();
                view(self.clone().into())
            }
        }
    };
}

impl_string_view!(&'static str);
impl_string_view!(String);
