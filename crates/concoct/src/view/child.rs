use crate::{Tree, View};
use std::{any::Any, cell::RefCell, rc::Rc};

/// Create a child view.
///
/// This type should be cloned and returned from a parent view to wrap its content.
pub fn child(view: impl View) -> Child<impl Tree> {
    Child {
        tree: Rc::new(RefCell::new(view.into_tree())),
    }
}

pub struct Child<T> {
    tree: Rc<RefCell<T>>,
}

impl<T> Clone for Child<T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
        }
    }
}

impl<T: Tree> View for Child<T> {
    fn into_tree(self) -> impl Tree {
        self
    }
}

impl<T: Tree> Tree for Child<T> {
    unsafe fn build(&mut self) {
        self.tree.borrow_mut().build()
    }

    unsafe fn rebuild(&mut self, last: &mut dyn Any) {
        let last = last.downcast_mut::<Self>().unwrap();
        self.tree.borrow_mut().rebuild(&mut *last.tree.borrow_mut())
    }

    unsafe fn remove(&mut self) {
        self.tree.borrow_mut().remove()
    }
}
