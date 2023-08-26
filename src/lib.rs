use std::{cell::RefCell, rc::Rc};
use web_sys::Document;

pub mod view;
use view::View;

pub struct Context {
    document: Document,
    stack: Vec<web_sys::Element>,
    pub update: Rc<RefCell<Option<Box<dyn FnMut()>>>>,
}

impl Context {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("HTML document missing body");

        Self {
            document,
            stack: vec![body.into()],
            update: Rc::new(RefCell::new(None)),
        }
    }
}

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run<T, V>(
        &mut self,
        state: T,
        update: impl Fn(&mut T) + 'static,
        f: impl Fn(&T) -> V + 'static,
    ) where
        T: 'static,
        V: View,
        V::State: 'static,
    {
        let f = Rc::new(f);

        let state = Rc::new(RefCell::new(state));
        let view_state: Rc<RefCell<Option<V::State>>> = Rc::new(RefCell::new(None));

        let cx_state = state.clone();
        let cx_f = f.clone();
        let cx_view_state = view_state.clone();

        let cx = Rc::new(RefCell::new(Context::new()));
        let update_cx = cx.clone();
        *cx.borrow_mut().update.borrow_mut() = Some(Box::new(move || {
            update(&mut cx_state.borrow_mut());

            let view = cx_f(&cx_state.borrow());
            view.rebuild(
                &mut update_cx.borrow_mut(),
                cx_view_state.borrow_mut().as_mut().unwrap(),
            );
        }));

        let view = f(&state.borrow());
        *view_state.borrow_mut() = Some(view.build(&mut cx.borrow_mut()));
    }
}
