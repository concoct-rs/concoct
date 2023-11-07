use std::{cell::RefCell, rc::Rc};
use generational_box::{Owner, Store};
use wasm_bindgen::JsCast;
use web_sys::Element;

pub mod html;

mod signal;
pub use signal::Signal;

mod view;
pub use view::View;

struct Scope {
    owner: Owner,
}

thread_local! {
    static STORE: Store = Store::default();

    static SCOPE: Scope = Scope { owner: STORE.try_with(|store| store.owner()).unwrap() };
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

