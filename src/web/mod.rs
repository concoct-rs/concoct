use crate::{
    hook::{use_context, use_provider, use_ref},
    Body, View,
};
use std::{borrow::Cow, cell::RefCell, rc::Rc};
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    Document, Element, Event, Node, Text, Window,
};

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

impl View for String {
    fn body(&self) -> impl Body {
        let web_cx = use_context::<WebContext>().unwrap();

        let data = use_ref(|| RefCell::new((self.clone(), None::<Text>)));
        let (last, node_cell) = &mut *data.borrow_mut();

        if let Some(node) = node_cell {
            if self != last {
                node.set_text_content(Some(&self));
                *last = self.clone();
            }
        } else {
            let node = web_cx.document.create_text_node(self);
            web_cx.parent.append_child(&node).unwrap();
            *node_cell = Some(node);
        }
    }
}
