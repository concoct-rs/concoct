#![cfg_attr(docsrs, feature(doc_cfg))]

use crate::{Context, ListenerData, Node, Signal};
use alloc::{
    boxed::Box,
    rc::{Rc, Weak},
    vec::Vec,
};
use core::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
};

/// A shared handle to an object.
///
/// The underlying object will be dropped when all handles to it are dropped.
pub struct Handle<O> {
    node: Rc<RefCell<Node>>,
    _marker: PhantomData<O>,
}

impl<O> Handle<O> {
    /// Start an object and create a new handle to it.
    pub(crate) fn new(object: O) -> Self
    where
        O: 'static,
    {
        Self {
            node: Rc::new(RefCell::new(Node {
                object: Box::new(object),
                listeners: Vec::new(),
                listening: Vec::new(),
            })),
            _marker: PhantomData,
        }
    }

    /// Bind a signal from this object to another object's slot.
    pub fn bind<O2, M>(&self, other: &Handle<O2>, slot: fn(&mut Context<O2>, M)) -> Binding
    where
        O: Signal<M>,
        O2: 'static,
        M: Clone + 'static,
    {
        let listener_id = slot.type_id();
        let listener = ListenerData {
            msg_id: TypeId::of::<M>(),
            listener_id: slot.type_id(),
            node: Rc::downgrade(&other.node),
            listen: |node, slot, msg: &dyn Any| {
                let slot = unsafe { *(slot as *const fn(&mut Context<O2>, &M)) };
                let handle = Handle {
                    node: node.upgrade().unwrap(),
                    _marker: PhantomData,
                };
                let mut cx = handle.cx();
                slot(&mut cx, msg.downcast_ref().unwrap())
            },
            slot: slot as _,
        };
        self.node.borrow_mut().listeners.push(listener);

        Binding {
            type_id: listener_id,
            node: Rc::downgrade(&self.node),
        }
    }

    /// Remove a binding from this object.
    pub fn unbind<O2, M>(&self, other: &Handle<O2>, slot: fn(&mut Context<O2>, M)) -> bool
    where
        O: Signal<M>,
        O2: 'static,
        M: Clone + 'static,
    {
        let mut node = self.node.borrow_mut();
        if let Some(idx) = node.listeners.iter().position(|listener_data| {
            listener_data.node.ptr_eq(&Rc::downgrade(&other.node))
                && listener_data.listener_id == slot.type_id()
        }) {
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
pub struct Binding {
    type_id: TypeId,
    node: Weak<RefCell<Node>>,
}

impl Binding {
    /// Remove this listener.
    pub fn unbind(&self) -> bool {
        let node_cell = self.node.upgrade().unwrap();
        let mut node = node_cell.borrow_mut();

        if let Some(idx) = node.listeners.iter().position(|listener_data| {
            listener_data.node.ptr_eq(&self.node) && listener_data.listener_id == self.type_id
        }) {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }
}
