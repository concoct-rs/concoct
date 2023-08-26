// #[cfg(feature = "gl")]
// mod renderer;
// #[cfg(feature = "gl")]
// pub use renderer::{Event, Renderer};

// pub mod view;

use std::num::NonZeroU64;

pub mod element;

pub mod view;

pub struct Id(NonZeroU64);

pub struct BuildContext {
    next_id: NonZeroU64,
    unused_ids: Vec<Id>,
}

impl Default for BuildContext {
    fn default() -> Self {
        Self {
            next_id: NonZeroU64::MIN,
            unused_ids: Vec::new(),
        }
    }
}

impl BuildContext {
    pub fn insert(&mut self) -> Id {
        self.unused_ids.pop().unwrap_or_else(|| {
            let id = Id(self.next_id);
            self.next_id = self.next_id.checked_add(1).unwrap();
            id
        })
    }

    pub fn remove(&mut self, id: Id) {
        self.unused_ids.push(id);
    }
}

use element::Element;
use view::View;
use web_sys::Document;

pub struct ElementContext {
    document: Document,
    stack: Vec<web_sys::Element>,
}

impl ElementContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("HTML document missing body");

        Self {
            document,

            stack: vec![body.into()],
        }
    }
}

pub struct App {
    build_cx: BuildContext,
    element_cx: ElementContext,
}

impl App {
    pub fn new() -> Self {
        Self {
            build_cx: BuildContext::default(),
            element_cx: ElementContext::new(),
        }
    }

    pub fn run(&mut self, view: impl View) {
        let (_id, _state, elem) = view.build(&mut self.build_cx);
        elem.build(&mut self.element_cx);
    }
}
