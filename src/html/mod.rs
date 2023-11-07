use crate::{runtime::Runtime, use_context, use_context_provider, Node, Scope, View};

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Element};

pub struct Parent(Element);

pub fn div() -> Div {
    Div::new()
}

#[derive(Clone)]
pub struct Div {
    view: Option<Rc<RefCell<dyn View>>>,
    on_click: Option<Rc<RefCell<dyn FnMut()>>>,
    callback: Option<Rc<Closure<dyn FnMut()>>>,
}

impl Div {
    pub fn new() -> Self {
        Self {
            view: None,
            on_click: None,
            callback: None,
        }
    }

    pub fn view(mut self, view: impl View + 'static) -> Self {
        self.view = Some(Rc::new(RefCell::new(view)));
        self
    }

    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Rc::new(RefCell::new(f)));
        self
    }
}

impl View for Div {
    fn view(&mut self) -> Option<Node> {
        let parent = use_context::<Parent>()
            .map(|cx| cx.0.clone())
            .unwrap_or_else(|| {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .unchecked_into()
            });

        let scope = Scope::current();
        let callback = scope
            .use_hook(|| {
                if let Some(f) = self.on_click.take() {
                    let closure: Closure<dyn FnMut()> =
                        Closure::wrap(Box::new(move || f.borrow_mut()()));
                    Some(Rc::new(closure))
                } else {
                    None
                }
            })
            .clone();
        drop(scope);

        let elem = use_context_provider(|| {
            let elem = window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("div")
                .unwrap();

            if let Some(f) = callback {
                elem.add_event_listener_with_callback("click", f.as_ref().as_ref().unchecked_ref())
                    .unwrap();
            }

            parent.append_child(&elem).unwrap();
            Parent(elem)
        })
        .0
        .clone();

        if let Some(view) = self.view.take() {
            Runtime::current().spawn(view)
        }

        Some(Node::Element(elem))
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl View for String {
    fn view(&mut self) -> Option<Node> {
        let parent = use_context::<Parent>()
            .map(|cx| cx.0.clone())
            .unwrap_or_else(|| {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .unchecked_into()
            });

        let elem = Scope::current()
            .use_hook(|| {
                let elem = window().unwrap().document().unwrap().create_text_node(self);
                parent.append_child(&elem).unwrap();
                Parent(elem.unchecked_into())
            })
            .0
            .clone();

        elem.set_text_content(Some(self));

        let document = web_sys::window().unwrap().document().unwrap();
        let elem = document.create_text_node(self);
        Some(Node::Element(elem.unchecked_into()))
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        None
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl View for &'static str {
    fn view(&mut self) -> Option<Node> {
        let parent = use_context::<Parent>()
            .map(|cx| cx.0.clone())
            .unwrap_or_else(|| {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .unchecked_into()
            });

        let elem = Scope::current()
            .use_hook(|| {
                let elem = window().unwrap().document().unwrap().create_text_node(self);
                parent.append_child(&elem).unwrap();
                Parent(elem.unchecked_into())
            })
            .0
            .clone();

        elem.set_text_content(Some(self));

        let document = web_sys::window().unwrap().document().unwrap();
        let elem = document.create_text_node(self);
        Some(Node::Element(elem.unchecked_into()))
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        None
    }

    fn remove(&mut self) {
        todo!()
    }
}
