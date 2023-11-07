use std::{cell::RefCell, rc::Rc};

use generational_box::{Owner, Store};
use wasm_bindgen::JsCast;
use web_sys::Element;

mod signal;
pub use signal::Signal;

struct Scope {
    owner: Owner,
}

thread_local! {
    static STORE: Store = Store::default();

    static SCOPE: Scope = Scope { owner: STORE.try_with(|store| store.owner()).unwrap() };
}

pub trait View {
    fn view(&mut self) -> Node;

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>>;

    fn remove(&mut self);
}

pub enum Node {
    Component(fn() -> Box<dyn View>),
    Element(Element),
}

pub fn run<V: View + 'static>(view: fn() -> V) {
    let mut stack: Vec<Rc<RefCell<Box<dyn View>>>> = vec![Rc::new(RefCell::new(Box::new(view())))];
    let document = web_sys::window().unwrap().document().unwrap();

    while let Some(mut view) = stack.pop() {
        let node = view.borrow_mut().view();
        if let Some(child) = view.borrow_mut().child() {
            stack.push(child);
        }
        match node {
            Node::Component(component) => stack.push(Rc::new(RefCell::new(component()))),
            Node::Element(elem) => {
                log::info!("elem");
                document.body().unwrap().append_child(&elem).unwrap();
            }
        }
    }
}

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
