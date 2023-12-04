use crate::{AnyView, IntoView, View};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct AnyChild {
    cell: Rc<RefCell<Option<Box<dyn FnOnce() -> Box<dyn AnyView>>>>>,
}

impl PartialEq for AnyChild {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl AnyChild {
    pub fn new(view: impl IntoView) -> Self {
        Self {
            cell: Rc::new(RefCell::new(Some(Box::new(|| Box::new(view.into_view()))))),
        }
    }
}

impl IntoView for AnyChild {
    fn into_view(self) -> impl View {
        self.cell.take().unwrap()().into_view()
    }
}
