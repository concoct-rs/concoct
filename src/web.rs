use crate::{hook::use_ref, Body, View};
use std::{borrow::Cow, cell::RefCell, rc::Rc};
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    Element, Event, Text,
};

pub fn div<C>(child: C) -> Div<C> {
    Div {
        handlers: Vec::new(),
        child: Rc::new(child),
    }
}

pub struct Div<C> {
    handlers: Vec<(Cow<'static, str>, Rc<RefCell<dyn FnMut(Event)>>)>,
    child: Rc<C>,
}

impl<C> Div<C> {
    pub fn on_click(mut self, handler: impl FnMut(Event) + 'static) -> Self {
        self.handlers
            .push((Cow::Borrowed("click"), Rc::new(RefCell::new(handler))));
        self
    }
}

#[derive(Default)]
struct Data {
    element: Option<Element>,
    callbacks: Vec<(
        Closure<dyn FnMut(Event)>,
        Rc<RefCell<Rc<RefCell<dyn FnMut(Event)>>>>,
    )>,
}

impl<C: View> View for Div<C> {
    fn body(&self) -> impl Body {
        let data = use_ref(|| RefCell::new(Data::default()));
        let mut data_ref = data.borrow_mut();

        if data_ref.element.is_none() {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let body = document.body().unwrap();

            let elem = document.create_element("div").unwrap();
            body.append_child(&elem).unwrap();

            for (name, handler) in &self.handlers {
                let handler = Rc::new(RefCell::new(handler.clone()));
                let handler_clone = handler.clone();
                let callback: Closure<dyn FnMut(Event)> = Closure::wrap(Box::new(move |event| {
                    handler.borrow_mut().borrow_mut()(event)
                }));
                elem.add_event_listener_with_callback(&name, callback.as_ref().unchecked_ref())
                    .unwrap();
                data_ref.callbacks.push((callback, handler_clone));
            }

            data_ref.element = Some(elem);
        } else {
            for ((_name, handler), (_callback, cell)) in
                self.handlers.iter().zip(&data_ref.callbacks)
            {
                *cell.borrow_mut() = handler.clone();
            }
        }

        self.child.clone()
    }
}

impl View for String {
    fn body(&self) -> impl Body {
        let data = use_ref(|| RefCell::new((self.clone(), None::<Text>)));
        let (last, node_cell) = &mut *data.borrow_mut();

        if let Some(node) = node_cell {
            if self != last {
                node.set_text_content(Some(&self));
                *last = self.clone();
            }
        } else {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let body = document.body().unwrap();

            let node = document.create_text_node(self);
            body.append_child(&node).unwrap();
            *node_cell = Some(node);
        }
    }
}
