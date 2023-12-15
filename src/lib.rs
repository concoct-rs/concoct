use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

struct ListenerData {
    msg_type_id: TypeId,
    listener_type_id: TypeId,
    f: Box<dyn FnMut(&dyn Any)>,
}

struct Node {
    object: Box<dyn Any>,
    listeners: Vec<ListenerData>,
}

pub struct Context<'a, O> {
    handle: Handle<O>,
    node: RefMut<'a, Node>,
}

impl<'a, O> Context<'a, O> {
    pub fn emit<M: 'static>(&mut self, msg: M) {
        for listener in &mut self.node.listeners {
            if listener.msg_type_id == msg.type_id() {
                (listener.f)(&msg)
            }
        }
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
    ) where
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

    pub fn listen<M: 'static>(&self, mut listener: impl FnMut(&M) + 'static) {
        let listener = ListenerData {
            msg_type_id: TypeId::of::<M>(),
            listener_type_id: listener.type_id(),
            f: Box::new(move |msg| listener(msg.downcast_ref().unwrap())),
        };
        self.node.borrow_mut().listeners.push(listener);
    }

    pub fn unlisten<M: 'static>(&self, listener: impl FnMut(&M) + 'static) -> bool {
        let mut node = self.node.borrow_mut();
        if let Some(idx) = node
            .listeners
            .iter()
            .position(|listener_data| listener_data.listener_type_id == listener.type_id())
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
            .position(|listener| listener.listener_type_id == self.type_id)
        {
            node.listeners.remove(idx);
            true
        } else {
            false
        }
    }
}
