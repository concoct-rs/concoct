use crate::{Tree, View};
use std::{cell::RefCell, rc::Rc};

pub struct Child<B> {
    cell: Rc<RefCell<Option<B>>>,
}

impl<B> Child<B> {
    pub fn new(body: B) -> Self {
        Self {
            cell: Rc::new(RefCell::new(Some(body))),
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
