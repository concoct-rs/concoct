use std::{rc::Rc, cell::RefCell};
use crate::Node;

pub trait View {
    fn view(&mut self) -> Node;

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>>;

    fn remove(&mut self);
}
