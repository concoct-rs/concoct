use crate::AnyComposable;
use std::{any::Any, cell::RefCell, rc::Rc};

pub struct Node {
    pub(crate) make_composable: Box<dyn FnMut() -> Box<dyn AnyComposable>>,
    pub(crate) composable: Option<Box<dyn AnyComposable>>,
    pub(crate) hooks: Rc<RefCell<Vec<Rc<RefCell<dyn Any>>>>>,
}
