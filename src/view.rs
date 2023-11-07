use crate::Node;
use std::{cell::RefCell, rc::Rc};

pub trait View {
    fn view(&mut self) -> Node;

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>>;

    fn remove(&mut self);
}

impl View for Box<dyn View> {
    fn view(&mut self) -> Node {
        (&mut **self).view()
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}
