use wasm_bindgen::JsCast;
use web_sys::window;

use crate::{html::Parent, use_context, Node, Scope};
use std::{cell::RefCell, rc::Rc};

pub trait View {
    fn view(&mut self) -> Option<Node>;

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>>;

    fn remove(&mut self);
}

impl<F, V> View for F
where
    F: FnMut() -> V + Clone + 'static,
    V: View + 'static,
{
    fn view(&mut self) -> Option<Node> {
        self().view()
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl View for Box<dyn View> {
    fn view(&mut self) -> Option<Node> {
        (&mut **self).view()
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl View for Rc<RefCell<dyn View>> {
    fn view(&mut self) -> Option<Node> {
        self.borrow_mut().view()
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl<A, B, C> View for (A, B, C)
where
    A: View + Clone + 'static,
    B: View + Clone + 'static,
    C: View + Clone + 'static,
{
    fn view(&mut self) -> Option<Node> {
        Some(Node::Components(vec![
            Box::new(self.0.clone()),
            Box::new(self.1.clone()),
            Box::new(self.2.clone()),
        ]))
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
        let parent = use_context::<Parent>()
            .map(|cx| cx.0.clone())
            .unwrap_or_else(|| {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .unchecked_into()
            });

        let elem = Scope::current()
            .use_hook(|| {
                let elem = window().unwrap().document().unwrap().create_text_node(self);
                parent.append_child(&elem).unwrap();
                Parent(elem.unchecked_into())
            })
            .0
            .clone();

        elem.set_text_content(Some(self));

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
        let parent = use_context::<Parent>()
            .map(|cx| cx.0.clone())
            .unwrap_or_else(|| {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .unchecked_into()
            });

        let elem = Scope::current()
            .use_hook(|| {
                let elem = window().unwrap().document().unwrap().create_text_node(self);
                parent.append_child(&elem).unwrap();
                Parent(elem.unchecked_into())
            })
            .0
            .clone();

        elem.set_text_content(Some(self));

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
