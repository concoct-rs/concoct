#![cfg_attr(docsrs, feature(doc_cfg))]

use crate::{Context, ListenerData, Node, Signal};
use alloc::{boxed::Box, rc::Rc};
use core::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
};

/// A shared handle to an object.
pub struct Handle<O> {
    node: Rc<RefCell<Node>>,
    _marker: PhantomData<O>,
}

impl<O> Handle<O> {
    /// Start an object and create a new handle to it.
    pub fn new(object: O) -> Self
    where
        O: 'static,
    {
        Self {
            node: Rc::new(RefCell::new(Node {
                object: Box::new(object),
                listeners: Default::default(),
            })),
            _marker: PhantomData,
        }
    }

    /// Bind a signal from this object to another object's slot.
    pub fn bind<O2, M>(&self, other: &Handle<O2>, slot: fn(&mut Context<O2>, M)) -> Listener
    where
        O: Signal<M>,
        O2: 'static,
        M: Clone + 'static,
    {
        let listener_id = slot.type_id();
        let listener = ListenerData {
            msg_id: TypeId::of::<M>(),
            listener_id: slot.type_id(),
            node: other.node.clone(),
            listen: |node, slot, msg: &dyn Any| {
                let slot = unsafe { *(slot as *const fn(&mut Context<O2>, &M)) };
                let handle = Handle {
                    node,
                    _marker: PhantomData,
                };
                let mut cx = handle.cx();
                slot(&mut cx, msg.downcast_ref().unwrap())
            },
            slot: slot as _,
        };
        self.node.borrow_mut().listeners.push(listener);

        Listener {
            type_id: listener_id,
            node: self.node.clone(),
        }
    }

    /// Remove a listener from this object.
    pub fn unlisten<M: 'static>(&self, listener: impl FnMut(&M) + 'static) -> bool
    where
        O: Signal<M>,
    {
        let mut node = self.node.borrow_mut();
        if let Some(idx) = node
            .listeners
            .iter()
            .position(|listener_data| listener_data.listener_id == listener.type_id())
        {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }

    /// Get the slot context for this object.
    pub fn cx(&self) -> Context<O> {
        Context {
            handle: self.clone(),
            node: Some(self.node.borrow_mut()),
        }
    }

    /// Borrow this object.
    pub fn borrow(&self) -> Ref<O>
    where
        O: 'static,
    {
        Ref::map(self.node.borrow(), |node| {
            node.object.downcast_ref().unwrap()
        })
    }

    /// Mutably borrow this object.
    pub fn borrow_mut(&self) -> RefMut<O>
    where
        O: 'static,
    {
        RefMut::map(self.node.borrow_mut(), |node| {
            node.object.downcast_mut().unwrap()
        })
    }
}

impl<O> Clone for Handle<O> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            _marker: self._marker.clone(),
        }
    }
}

/// A handle to a listener for an object's signal.
pub struct Listener {
    type_id: TypeId,
    node: Rc<RefCell<Node>>,
}

impl Listener {
    /// Remove this listener.
    pub fn unlisten(&self) -> bool {
        let mut node = self.node.borrow_mut();

        if let Some(idx) = node
            .listeners
            .iter()
            .position(|listener_data| listener_data.listener_id == self.type_id)
        {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }
}
