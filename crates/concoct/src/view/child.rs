use crate::{Tree, View};
use std::{cell::RefCell, rc::Rc};

/// Child view.
///
/// This type should be cloned and returned from a parent view to wrap its content.
///
/// ## Panics
/// This view can only be used once, then it will panic.
pub struct Child<V> {
    cell: Rc<RefCell<Option<V>>>,
}

impl<B> Child<B> {
    pub fn new(view: B) -> Self {
        Self {
            cell: Rc::new(RefCell::new(Some(view))),
        }
    }
}

impl<B> Clone for Child<B> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<B: View> View for Child<B> {
    fn into_tree(self) -> impl Tree {
        self.cell.take().unwrap().into_tree()
    }
}
