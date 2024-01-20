use crate::{Tree, View};
use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

trait AnyTree {
    fn as_any(&mut self) -> &mut dyn Any;

    fn as_tree(&mut self) -> &mut dyn Tree;
}

impl<T: Tree> AnyTree for T {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn as_tree(&mut self) -> &mut dyn Tree {
        self
    }
}

/// Child view.
///
/// This type should be cloned and returned from a parent view to wrap its content.
pub struct Child<V> {
    tree: Rc<RefCell<dyn AnyTree>>,
    _marker: PhantomData<V>,
}

impl<V: View> Child<V> {
    pub fn new(view: V) -> Self {
        Self {
            tree: Rc::new(RefCell::new(view.into_tree())),
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Child<T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            _marker: PhantomData,
        }
    }
}

impl<V: View> View for Child<V> {
    fn into_tree(self) -> impl Tree {
        self
    }
}

impl<V: View> Tree for Child<V> {
    unsafe fn build(&mut self) {
        self.tree.borrow_mut().as_tree().build()
    }

    unsafe fn rebuild(&mut self, last: &mut dyn Any) {
        let last = last.downcast_mut::<Self>().unwrap();
        self.tree
            .borrow_mut()
            .as_tree()
            .rebuild(&mut *last.tree.borrow_mut().as_any())
    }

    unsafe fn remove(&mut self) {
        self.tree.borrow_mut().as_tree().remove()
    }
}
