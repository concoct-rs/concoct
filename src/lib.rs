use generational_box::Store;
use runtime::Runtime;

use web_sys::Element;

mod html;
pub use html::Html;

pub mod runtime;

mod scope;
pub use scope::Scope;

mod signal;
pub use signal::{use_signal, Signal};

mod view;
pub use view::View;

mod use_context;
pub use use_context::{use_context, use_context_provider, UseContext};

thread_local! {
 static STORE: Store = Store::default();

}

pub enum Node {
    Component(Box<dyn View>),
    Element(Element),
    Components(Vec<Box<dyn View>>),
}

pub fn run<V: View + 'static>(component: fn() -> V) {
    Runtime::default().enter();

    Runtime::current().spawn(component);

    for _ in 0..10 {
        Runtime::current().poll();
    }
}
