use crate::{IntoView, View};
use std::{cell::RefCell, rc::Rc};

pub struct Child<C> {
    cell: Rc<RefCell<Option<C>>>,
}

impl<C> Child<C> {
    pub fn new(view: C) -> Self {
        Self {
            cell: Rc::new(RefCell::new(Some(view))),
        }
    }
}

impl<C> Clone for Child<C> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<C: IntoView> IntoView for Child<C> {
    fn into_view(self) -> impl View {
        self.cell.take().unwrap().into_view()
    }
}
