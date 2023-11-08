use generational_box::Store;
use web_sys::Element;

mod event;
pub use self::event::{InputEvent, MouseEvent};

mod html;
pub use html::Html;

pub mod runtime;
pub use self::runtime::Runtime;

mod scope;
pub use scope::Scope;

mod signal;
pub use signal::{use_signal, Signal};

mod view;
pub use view::View;

mod use_context;
pub use use_context::{use_context, use_context_provider, UseContext};

mod use_hook;
pub use use_hook::use_hook;

thread_local! {
 static STORE: Store = Store::default();

}

pub enum Node {
    Component(Box<dyn View>),
    Element(Element),
    Components(Vec<Box<dyn View>>),
}

pub fn run(view: impl View + 'static) {
    Runtime::default().enter();

    Runtime::current().spawn(view);

    for _ in 0..10 {
        Runtime::current().poll();
    }
}
