use crate::{Handle, Node, Signal};
use core::{
    cell::RefMut,
    ops::{Deref, DerefMut},
};

/// A mutable context to an object.
pub struct Context<'a, O> {
    pub(crate) handle: Handle<O>,
    pub(crate) node: RefMut<'a, Node>,
}

impl<'a, O> Context<'a, O> {
    /// Emit a message from this object.
    pub fn emit<M>(&mut self, msg: M)
    where
        O: Signal<M>,
        M: 'static,
    {
        O::emit(self, msg)
    }

    /// Get a handle to this object.
    pub fn handle(&self) -> Handle<O> {
        self.handle.clone()
    }
}

impl<O: 'static> Deref for Context<'_, O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        self.node.object.downcast_ref().unwrap()
    }
}

impl<O: 'static> DerefMut for Context<'_, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node.object.downcast_mut().unwrap()
    }
}
