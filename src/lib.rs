// #[cfg(feature = "gl")]
// mod renderer;
// #[cfg(feature = "gl")]
// pub use renderer::{Event, Renderer};

// pub mod view;

use std::{cell::RefCell, num::NonZeroU64, rc::Rc};

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
    pub update: Rc<RefCell<dyn FnMut()>>,
}

impl ElementContext {
    pub fn new(update: impl FnMut() + 'static) -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("HTML document missing body");

        Self {
            document,
            stack: vec![body.into()],
            update: Rc::new(RefCell::new(update)),
        }
    }
}

pub struct App {
    build_cx: BuildContext,
}

impl App {
    pub fn new() -> Self {
        Self {
            build_cx: BuildContext::default(),
        }
    }

    pub fn run<T, V>(
        &mut self,
        mut state: T,
        update: impl Fn(&mut T) + 'static,
        f: impl Fn(&T) -> V + 'static,
    ) where
        T: 'static,
        V: View,
        <V::Element as Element>::State: 'static,
    {
        let f = Rc::new(f);

        let state = Rc::new(RefCell::new(state));
        let element_state = Rc::new(RefCell::new(None));

        let cx_state = state.clone();
        let cx_f = f.clone();
        let cx_element_state = element_state.clone();

        let mut cx = ElementContext::new(move || {
            update(&mut cx_state.borrow_mut());

            let mut view = cx_f(&cx_state.borrow());

            &cx_element_state;
        });

        let (_id, state, elem) = f(&state.borrow()).build(&mut self.build_cx);
        *element_state.borrow_mut() = Some(elem.build(&mut cx));
    }
}
