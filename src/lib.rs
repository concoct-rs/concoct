use generational_box::{Owner, Store};
use runtime::Runtime;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::Element;

pub mod html;

pub mod runtime;

mod scope;
pub use scope::Scope;

mod signal;
pub use signal::Signal;

mod view;
pub use view::View;

thread_local! {
 static STORE: Store = Store::default();

}

pub enum Node {
    Component(fn() -> Box<dyn View>),
    Element(Element),
}

pub fn run<V: View + 'static>(component: fn() -> V) {
    Runtime::default().enter();

    Runtime::current().spawn(component);

    Runtime::current().poll();
}
