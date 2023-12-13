use crate::{object::AnyObject, Handle, Object, Runtime, Slot};
use alloc::rc::Rc;
use core::{
    any::{Any, TypeId},
    cell::RefCell,
    marker::PhantomData,
};
use slotmap::DefaultKey;

/// Handle to an object's signal for a specific message.
pub struct SignalHandle<M> {
    pub(crate) make_emit: Rc<dyn Fn() -> Box<dyn FnOnce(&mut dyn AnyObject, DefaultKey, &dyn Any)>>,
    pub(crate) key: DefaultKey,
    pub(crate) _marker: PhantomData<M>,
}

impl<M> Clone for SignalHandle<M> {
    fn clone(&self) -> Self {
        Self {
            make_emit: self.make_emit.clone(),
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
        crate::Runtime::current().inner.borrow_mut().channel.send(
            crate::rt::RuntimeMessage(crate::rt::RuntimeMessageKind::Emit {
                key,
                msg: Box::new(msg),
                f: (self.make_emit)(),
            },
        ));
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
