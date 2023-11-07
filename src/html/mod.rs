use crate::{runtime::Runtime, Node, View};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;

pub fn div() -> Div {
    Div::new()
}

pub struct Div {
    view: Option<Box<dyn View>>,
}

impl Div {
    pub fn new() -> Self {
        Self { view: None }
    }

    pub fn view(mut self, mut view: impl View + 'static) -> Self {
        self.view = Some(Box::new(view));
        self
    }

    pub fn on_click(self, _f: impl FnMut() + 'static) -> Self {
        self
    }
}

impl View for Div {
    fn view(&mut self) -> Option<Node> {
        let document = web_sys::window().unwrap().document().unwrap();
        let elem = document.create_element("div").unwrap();

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
        log::info!("{:?}", &self);

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
        log::info!("{:?}", &self);

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
