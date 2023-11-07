use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use crate::{View, Node};

pub struct Div {
    view: Option<Rc<RefCell<Box<dyn View>>>>,
}

impl Div {
    pub fn new() -> Self {
        Self { view: None }
    }

    pub fn view(mut self, view: impl View + 'static) -> Self {
        self.view = Some(Rc::new(RefCell::new(Box::new(view))));
        self
    }
}

impl View for Div {
    fn view(&mut self) -> Node {
        let document = web_sys::window().unwrap().document().unwrap();
        let elem = document.create_element("div").unwrap();

        Node::Element(elem)
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        self.view.clone()
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl View for String {
    fn view(&mut self) -> Node {
        let document = web_sys::window().unwrap().document().unwrap();
        let elem = document.create_text_node(self);
        Node::Element(elem.unchecked_into())
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        None
    }

    fn remove(&mut self) {
        todo!()
    }
}
