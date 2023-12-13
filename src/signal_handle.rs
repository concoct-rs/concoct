use crate::{Handle, Object, Runtime, Slot};
use slotmap::DefaultKey;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
};

pub struct SignalHandle<M> {
    pub(crate) key: DefaultKey,
    pub(crate) _marker: PhantomData<M>,
}

impl<M> Clone for SignalHandle<M> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<M> SignalHandle<M> {
    pub fn emit(&self, msg: M)
    where
        M: 'static,
    {
        let key = self.key;
        Runtime::current()
            .tx
            .unbounded_send(crate::rt::RuntimeMessage::Signal {
                key,
                msg: Box::new(msg),
            })
            .unwrap();
    }

    pub fn listen(&self, mut f: impl FnMut(&M) + 'static)
    where
        M: 'static,
    {
        Runtime::current().inner.borrow_mut().listeners.insert(
            (self.key, TypeId::of::<M>()),
            vec![Rc::new(RefCell::new(move |msg: &dyn Any| {
                f(msg.downcast_ref().unwrap())
            }))],
        );
    }

    pub fn bind(&self, other: &Handle<impl Object + Slot<M> + 'static>)
    where
        M: Clone + 'static,
    {
        let other = other.clone();

        self.listen(move |msg: &M| {
            other.send(msg.clone());
        });
    }
}
