use crate::{Handler, Object, Runtime, Signal};
use futures::channel::mpsc;
use slotmap::DefaultKey;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
};

pub struct Context<T: ?Sized> {
    key: DefaultKey,
    _marker: PhantomData<T>,
}

impl<T> Clone for Context<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<T> Context<T> {
    pub(crate) fn new(key: DefaultKey) -> Self {
        Self {
            key,
            _marker: PhantomData,
        }
    }

    pub fn send<M>(&self, msg: M)
    where
        T: Handler<M> + 'static,
        M: 'static,
    {
        let key = self.key;
        Runtime::current()
            .tx
            .unbounded_send(crate::rt::RuntimeMessage::Handle {
                key,
                f: Box::new(move |any_task| {
                    let task = any_task.as_any_mut().downcast_mut::<T>().unwrap();
                    task.handle(Context::new(key), msg);
                }),
            })
            .unwrap();
    }

    pub fn listen<M: 'static>(&self, mut f: impl FnMut(&M) + 'static) {
        Runtime::current().inner.borrow_mut().listeners.insert(
            (self.key, TypeId::of::<M>()),
            vec![Rc::new(RefCell::new(move |msg: &dyn Any| {
                f(msg.downcast_ref().unwrap())
            }))],
        );
    }

    pub fn bind<M, T2>(&self, other: &Context<T2>)
    where
        M: Clone + 'static,
        T2: Object + Handler<M> + 'static,
    {
        let other = other.clone();

        self.listen(move |msg: &M| {
            other.send(msg.clone());
        });
    }

    pub fn channel<M>(&self) -> mpsc::UnboundedReceiver<M>
    where
        M: Clone + 'static,
    {
        let (tx, rx) = mpsc::unbounded();
        self.listen(move |msg: &M| {
            tx.unbounded_send(msg.clone()).unwrap();
        });
        rx
    }

    pub fn emit<M>(&self, msg: M)
    where
        T: Signal<M> + 'static,
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
}
