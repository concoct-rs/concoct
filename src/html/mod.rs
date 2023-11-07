use crate::{runtime::Runtime, Node, View};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;

pub struct Div {
    child: Option<Box<dyn FnMut() -> Box<dyn View>>>,
}

impl Div {
    pub fn new() -> Self {
        Self { child: None }
    }

    pub fn child<V: View + 'static>(mut self, mut component: impl FnMut() -> V + 'static) -> Self {
        self.child = Some(Box::new(move || Box::new(component())));
        self
    }
}

impl View for Div {
    fn view(&mut self) -> Node {
        let document = web_sys::window().unwrap().document().unwrap();
        let elem = document.create_element("div").unwrap();

        if let Some(component) = self.child.take() {
            Runtime::current().spawn(component)
        }

        Node::Element(elem)
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl View for String {
    fn view(&mut self) -> Node {
        log::info!("{:?}", &self);

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
