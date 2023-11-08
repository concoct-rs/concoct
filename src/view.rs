use crate::{html::Parent, use_context, use_hook, Runtime};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::window;

pub trait View {
    fn view(&mut self);

    fn remove(&mut self);
}

impl<F, V> View for F
where
    F: FnMut() -> V + Clone + 'static,
    V: View + 'static,
{
    fn view(&mut self) {
        self().view()
    }

    fn remove(&mut self) {
        self().remove()
    }
}

impl View for Box<dyn View> {
    fn view(&mut self) {
        (&mut **self).view()
    }

    fn remove(&mut self) {
        (&mut **self).remove()
    }
}

impl View for Rc<RefCell<dyn View>> {
    fn view(&mut self) {
        self.borrow_mut().view()
    }

    fn remove(&mut self) {
        self.borrow_mut().remove()
    }
}

impl View for () {
    fn view(&mut self) {
        
    }

    fn remove(&mut self) {
       
    }
}

impl<A, B, C> View for (A, B, C)
where
    A: View + Clone + 'static,
    B: View + Clone + 'static,
    C: View + Clone + 'static,
{
    fn view(&mut self) {
        Runtime::current().spawn(self.0.clone());
        Runtime::current().spawn(self.1.clone());
        Runtime::current().spawn(self.2.clone());
    }

    fn remove(&mut self) {
        self.0.remove();
        self.1.remove();
        self.2.remove();
    }
}

impl View for String {
    fn view(&mut self) {
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

        let elem = use_hook(|| {
            let elem = window().unwrap().document().unwrap().create_text_node(self);
            parent.append_child(&elem).unwrap();
            Parent(elem.unchecked_into())
        })
        .0
        .clone();

        elem.set_text_content(Some(self));

        let document = web_sys::window().unwrap().document().unwrap();
        document.create_text_node(self);
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl View for &'static str {
    fn view(&mut self) {
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

        let elem = use_hook(|| {
            let elem = window().unwrap().document().unwrap().create_text_node(self);
            parent.append_child(&elem).unwrap();
            Parent(elem.unchecked_into())
        })
        .0
        .clone();

        elem.set_text_content(Some(self));

        let document = web_sys::window().unwrap().document().unwrap();
        document.create_text_node(self);
    }

    fn remove(&mut self) {
        todo!()
    }
}
