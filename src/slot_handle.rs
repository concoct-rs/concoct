use crate::{object::AnyObject, Runtime};
use slotmap::DefaultKey;
use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

/// Handle to an object's slot for a specific message.
pub struct SlotHandle<M> {
    pub(crate) key: DefaultKey,
    pub(crate) f: Rc<RefCell<dyn FnMut(&mut dyn AnyObject, Box<dyn Any>)>>,
    pub(crate) _marker: PhantomData<M>,
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
                        f.borrow_mut()(any_object, Box::new(msg));
                    }),
                },
            ))
            .unwrap();
    }
}
