use crate::Node;
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

impl<A: View +Clone + 'static, B: View +Clone+ 'static, C: View +Clone+ 'static> View for (A, B, C) {
    fn view(&mut self) -> Option<Node> {
        Some(Node::Components(vec![
           Box::new( self.0.clone()),
           Box::new( self.1.clone()),
           Box::new( self.2.clone()),
        ]))
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}
