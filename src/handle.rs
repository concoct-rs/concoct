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
    marker::PhantomData, mem,
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
        Self::from_node(Rc::new(RefCell::new(Node {
            object: Box::new(object),
            listeners: Vec::new(),
            bindings: Vec::new(),
        })))
    }

    fn from_node(node: Rc<RefCell<Node>>) -> Self {
        Self {
            node,
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
        let slot_id = slot.type_id();
        let listener = ListenerData {
            msg_id: TypeId::of::<M>(),
            slot_id,
            node: Rc::downgrade(&other.node),
            listen: |node, slot, msg: &dyn Any| {
                let slot: fn(&mut Context<O2>, M)  = unsafe { mem::transmute(slot) };
                if let Some(node) = node.upgrade() {
                    let handle = Handle::from_node(node);
                    let mut cx = handle.cx();
                    slot(&mut cx, msg.downcast_ref::<M>().unwrap().clone())
                }
            },
            slot: slot as _,
        };
        self.node.borrow_mut().listeners.push(listener);

        Binding {
            slot_id,
            node: Rc::downgrade(&self.node),
        }
    }

    /// Remove a binding from this object.
    /// This function returns `true` if the object previously had this listener attached.
    ///
    /// ```
    /// use concoct::{Object, Signal};
    ///
    /// struct App;
    ///
    /// impl App {
    ///     fn set_value(cx: &mut concoct::Context<Self>, value: i32) {}
    /// }
    ///
    /// impl Object for App {}
    ///
    /// impl Signal<i32> for App {}
    ///
    /// let object_a = App.start();
    /// let object_b = App.start();
    ///
    /// object_a.bind(&object_b, App::set_value);
    ///
    /// assert!(object_a.unbind(App::set_value));
    /// ```
    pub fn unbind<O2, M>(&self, slot: fn(&mut Context<O2>, M)) -> bool
    where
        O: Signal<M>,
        O2: 'static,
        M: Clone + 'static,
    {
        let mut node = self.node.borrow_mut();

        // Check if this slot exists as a listener for this object.
        if let Some(idx) = node
            .listeners
            .iter()
            .position(|listener| listener.slot_id == slot.type_id())
        {
            // Remove this listener.
            let listener = node.listeners.remove(idx);

            // Remove the binding to this listener from the bound object.
            if let Some(listener_node) = listener.node.upgrade() {
                let mut listener_node = listener_node.borrow_mut();
                if let Some(idx) = listener_node
                    .bindings
                    .iter()
                    .position(|binding| binding.slot_id == slot.type_id())
                {
                    listener_node.bindings.remove(idx).unbind();
                }
            }

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
            _marker: self._marker,
        }
    }
}

/// A handle to a listener for an object's signal.
pub struct Binding {
    slot_id: TypeId,
    node: Weak<RefCell<Node>>,
}

impl Binding {
    /// Remove this listener.
    pub fn unbind(&self) -> bool {
        let node_cell = self.node.upgrade().unwrap();
        let mut node = node_cell.borrow_mut();

        if let Some(idx) = node.listeners.iter().position(|listener_data| {
            listener_data.node.ptr_eq(&self.node) && listener_data.slot_id == self.slot_id
        }) {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }
}
