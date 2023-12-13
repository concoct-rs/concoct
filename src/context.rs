use slotmap::DefaultKey;
use std::marker::PhantomData;

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
    cfg_rt!(
        pub(crate) fn new(key: DefaultKey) -> Self {
            Self {
                key,
                _marker: PhantomData,
            }
        }

        pub(crate) fn from_handle(handle: &crate::Handle<T>) -> Self {
            Self::new(handle.dropper.key)
        }

        pub fn send<M>(&self, msg: M)
        where
            T: crate::Slot<M> + 'static,
            M: 'static,
        {
            let key = self.key;
            crate::Runtime::current()
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

        pub fn listen<M>(&self, mut f: impl FnMut(&M) + 'static)
        where
            T: crate::Signal<M>,
            M: 'static,
        {
            crate::Runtime::current()
                .inner
                .borrow_mut()
                .listeners
                .insert(
                    (self.key, std::any::TypeId::of::<M>()),
                    vec![std::rc::Rc::new(std::cell::RefCell::new(
                        move |msg: &dyn std::any::Any| f(msg.downcast_ref().unwrap()),
                    ))],
                );
        }

        pub fn bind<M>(&self, other: &Context<impl crate::Object + crate::Slot<M> + 'static>)
        where
            T: crate::Signal<M>,
            M: Clone + 'static,
        {
            let other = other.clone();

            self.listen(move |msg: &M| {
                other.send(msg.clone());
            });
        }

        pub fn channel<M>(&self) -> futures::channel::mpsc::UnboundedReceiver<M>
        where
            T: crate::Signal<M>,
            M: Clone + 'static,
        {
            let (tx, rx) = futures::channel::mpsc::unbounded();
            self.listen(move |msg: &M| {
                tx.unbounded_send(msg.clone()).unwrap();
            });
            rx
        }

        pub fn emit<M>(&self, msg: M)
        where
            T: crate::Signal<M> + 'static,
            M: 'static,
        {
            let key = self.key;
            crate::Runtime::current()
                .tx
                .unbounded_send(crate::rt::RuntimeMessage::Signal {
                    key,
                    msg: Box::new(msg),
                })
                .unwrap();
        }

        pub fn signal<M>(&self) -> crate::SignalHandle<M> {
            let key = self.key;
            crate::SignalHandle {
                key: key,
                _marker: PhantomData,
            }
        }

        pub fn slot<M>(&self) -> crate::SlotHandle<M>
        where
            T: crate::Slot<M> + 'static,
            M: 'static,
        {
            let key = self.key;
            crate::SlotHandle {
                key,
                f: std::rc::Rc::new(std::cell::RefCell::new(
                    move |any_task: &mut dyn crate::rt::AnyTask, msg: Box<dyn std::any::Any>| {
                        let task = any_task.as_any_mut().downcast_mut::<T>().unwrap();
                        task.handle(Context::new(key), *msg.downcast().unwrap());
                    },
                )),
                _marker: PhantomData,
            }
        }
    );
}
