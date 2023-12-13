use crate::{object::AnyObject, Runtime};
use slotmap::DefaultKey;
use std::{any::Any, marker::PhantomData, sync::Arc};

/// Handle to an object's slot for a specific message.
pub struct SlotHandle<M> {
    pub(crate) key: DefaultKey,
    pub(crate) f: Arc<dyn Fn(&mut dyn AnyObject, Box<dyn Any>) + Send + Sync>,
    pub(crate) _marker: PhantomData<M>,
}

impl<M> SlotHandle<M> {
    pub fn send(&self, msg: M)
    where
        M: 'static,
    {
        let key = self.key;
        let f = self.f.clone();
        Runtime::current()
            .tx
            .unbounded_send(crate::rt::RuntimeMessage(
                crate::rt::RuntimeMessageKind::Handle {
                    key,
                    f: Box::new(move |any_object| {
                        f(any_object, Box::new(msg));
                    }),
                },
            ))
            .unwrap();
    }
}

impl<M> Clone for SlotHandle<M> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            f: self.f.clone(),
            _marker: self._marker.clone(),
        }
    }
}

unsafe impl<M> Send for SlotHandle<M> {}

unsafe impl<M> Sync for SlotHandle<M> {}

impl<M> Unpin for SlotHandle<M> {}
