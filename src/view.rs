use crate::Node;
use std::{cell::RefCell, rc::Rc};

pub trait View {
    fn view(&mut self) -> Option<Node>;

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>>;

    fn remove(&mut self);
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

impl<A: View, B: View, C: View> View for (A, B, C) {
    fn view(&mut self) -> Option<Node> {
        self.0.view();
        self.1.view();
        self.2.view();
        None
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}
