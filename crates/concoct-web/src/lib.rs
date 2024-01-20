use concoct::{
    hook::{use_context, use_on_drop, use_provider, use_ref},
    Body, TextViewContext, View,
};
use std::{cell::RefCell, rc::Rc};
use web_sys::{Document, Node, Text, Window};

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

            let data_clone = data.clone();
            use_on_drop(move || {
                if let Some(node) = &data_clone.borrow_mut().1 {
                    node.remove();
                }
            });

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

pub async fn run(view: impl View) {
    concoct::run(WebRoot {
        body: Rc::new(view),
    }).await;
   
}
