use crate::{rt::AnyTask, Runtime};
use slotmap::DefaultKey;
use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

pub struct SlotHandle<M> {
    pub(crate) key: DefaultKey,
    pub(crate) f: Rc<RefCell<dyn FnMut(&mut dyn AnyTask, Box<dyn Any>)>>,
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
        .inner.borrow_mut().channel
        .send(crate::rt::RuntimeMessage::Handle {
                key,
                f: Box::new(move |any_task| {
                    f.borrow_mut()(any_task, Box::new(msg));
                }),
            });
    }
}
