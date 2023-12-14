use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
};

struct ListenerData {
    type_id: TypeId,
    f: Box<dyn FnMut(&dyn Any)>,
}

struct Node {
    object: Box<dyn Any>,
    listeners: Vec<ListenerData>,
}

pub struct Handle<O> {
    node: Rc<RefCell<Node>>,
    _marker: PhantomData<O>,
}

impl<O> Handle<O> {
    pub fn listen<M: 'static>(&self, mut listener: impl FnMut(&M) + 'static) {
        let listener = ListenerData {
            type_id: listener.type_id(),
            f: Box::new(move |msg| listener(msg.downcast_ref().unwrap())),
        };
        self.node.borrow_mut().listeners.push(listener);
    }

    pub fn unlisten<M: 'static>(&self, listener: impl FnMut(&M) + 'static) -> bool {
        let mut node = self.node.borrow_mut();
        if let Some(idx) = node
            .listeners
            .iter()
            .position(|listener_data| listener_data.type_id == listener.type_id())
        {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn borrow(&self) -> Ref<O>
    where
        O: 'static,
    {
        Ref::map(self.node.borrow(), |node| {
            node.object.downcast_ref().unwrap()
        })
    }

    pub fn borrow_mut(&self) -> RefMut<O>
    where
        O: 'static,
    {
        RefMut::map(self.node.borrow_mut(), |node| {
            node.object.downcast_mut().unwrap()
        })
    }
}

pub struct Listener {
    type_id: TypeId,
    node: Rc<RefCell<Node>>,
}

impl Listener {
    pub fn unlisten(&self) -> bool {
        let mut node = self.node.borrow_mut();
        if let Some(idx) = node
            .listeners
            .iter()
            .position(|listener| listener.type_id == self.type_id)
        {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }
}
