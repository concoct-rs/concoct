use crate::{
    handle::{HandleGuard, LISTENER_ID},
    object::AnyObject,
    rt::RuntimeMessage,
    Handle, Object, Slot,
};
use slotmap::DefaultKey;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::atomic::Ordering,
};

/// Handle to an object's signal for a specific message.
pub struct SignalHandle<M> {
    pub(crate) handle: HandleGuard,
    pub(crate) make_emit: Rc<dyn Fn() -> Box<dyn FnOnce(&mut dyn AnyObject, DefaultKey, &dyn Any)>>,
    pub(crate) make_listen: Rc<dyn Fn() -> Box<dyn FnOnce(&mut dyn AnyObject)>>,
    pub(crate) _marker: PhantomData<M>,
}

impl<M> Clone for SignalHandle<M> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            make_emit: self.make_emit.clone(),
            make_listen: self.make_listen.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<M> SignalHandle<M> {
    pub fn emit(&self, msg: M)
    where
        M: 'static,
    {
        let key = self.handle.inner.key;
        self.handle
            .inner
            .tx
            .unbounded_send(crate::rt::RuntimeMessage(
                crate::rt::RuntimeMessageKind::Emit {
                    key,
                    msg: Box::new(msg),
                    f: (self.make_emit)(),
                },
            ))
            .unwrap();
    }

    pub fn listen(&self, mut f: impl FnMut(&M) + 'static)
    where
        M: 'static,
    {
        let _cx = self.handle.clone();
        self.handle
            .inner
            .tx
            .unbounded_send(RuntimeMessage(crate::rt::RuntimeMessageKind::Listen {
                id: LISTENER_ID.fetch_add(1, Ordering::SeqCst),
                key: self.handle.inner.key,
                type_id: TypeId::of::<M>(),
                f: Rc::new(RefCell::new(move |msg: &dyn Any| {
                    f(msg.downcast_ref().unwrap())
                })),
                listen_f: (self.make_listen)(),
                listener: None,
            }))
            .unwrap();
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
