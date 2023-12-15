use crate::{Context, ListenerData, Node, Signal};
use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
};

pub struct Handle<O> {
    node: Rc<RefCell<Node>>,
    _marker: PhantomData<O>,
}

impl<O> Handle<O> {
    pub fn new(object: O) -> Self
    where
        O: 'static,
    {
        Self {
            node: Rc::new(RefCell::new(Node {
                object: Box::new(object),
                listeners: Vec::new(),
            })),
            _marker: PhantomData,
        }
    }

    pub fn bind<O2, M>(
        &self,
        other: &Handle<O2>,
        mut listener: impl FnMut(&mut Context<O2>, M) + 'static,
    ) -> Listener
    where
        O: Signal<M>,
        O2: 'static,
        M: Clone + 'static,
    {
        let other = other.clone();
        self.listen::<M>(move |msg| {
            let mut cx = Context {
                handle: other.clone(),
                node: other.node.borrow_mut(),
            };
            listener(&mut cx, msg.clone());
        })
    }

    pub fn listen<M: 'static>(&self, mut listener: impl FnMut(&M) + 'static) -> Listener
    where
        O: Signal<M>,
    {
        let listener_id = listener.type_id();
        let listener = ListenerData {
            msg_id: TypeId::of::<M>(),
            listener_id: listener.type_id(),
            f: Box::new(move |msg| listener(msg.downcast_ref().unwrap())),
        };
        self.node.borrow_mut().listeners.push(listener);

        Listener {
            type_id: listener_id,
            node: self.node.clone(),
        }
    }

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

    pub fn cx(&self) -> Context<O> {
        Context {
            handle: self.clone(),
            node: self.node.borrow_mut(),
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

impl<O> Clone for Handle<O> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            _marker: self._marker.clone(),
        }
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
            .position(|listener| listener.listener_id == self.type_id)
        {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }
}
