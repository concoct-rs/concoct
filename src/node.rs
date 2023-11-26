use crate::AnyView;
use std::{any::Any, cell::RefCell, rc::Rc};

pub struct Node {
    pub(crate) make_composable: Box<dyn FnMut() -> Box<dyn AnyView>>,
    pub(crate) composable: Option<Rc<RefCell<Box<dyn AnyView>>>>,
    pub(crate) hooks: Rc<RefCell<Vec<Rc<RefCell<dyn Any>>>>>,
}
