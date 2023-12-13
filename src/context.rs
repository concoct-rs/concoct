use core::marker::PhantomData;
use slotmap::DefaultKey;

pub struct Context<O: ?Sized> {
    key: DefaultKey,
    _marker: PhantomData<O>,
}

impl<O> Clone for Context<O> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<O> Context<O> {
    cfg_rt!(
        pub(crate) fn new(key: DefaultKey) -> Self {
            Self {
                key,
                _marker: PhantomData,
            }
        }

        pub(crate) fn from_handle(handle: &crate::Handle<O>) -> Self {
            Self::new(handle.dropper.key)
        }

        pub fn send<M>(&self, msg: M)
        where
            O: crate::Slot<M> + 'static,
            M: 'static,
        {
            let key = self.key;
            crate::Runtime::current().inner.borrow_mut().channel.send(
                crate::rt::RuntimeMessage::Handle {
                    key,
                    f: Box::new(move |any_object| {
                        let object = any_object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.handle(Context::new(key), msg);
                    }),
                },
            )
        }

        pub fn listen<M>(&self, mut f: impl FnMut(&M) + 'static)
        where
            O: crate::Signal<M>,
            M: 'static,
        {
            crate::Runtime::current()
                .inner
                .borrow_mut()
                .listeners
                .insert(
                    (self.key, core::any::TypeId::of::<M>()),
                    vec![alloc::rc::Rc::new(core::cell::RefCell::new(
                        move |msg: &dyn core::any::Any| f(msg.downcast_ref().unwrap()),
                    ))],
                );
        }

        pub fn bind<M>(&self, other: &Context<impl crate::Object + crate::Slot<M> + 'static>)
        where
            O: crate::Signal<M>,
            M: Clone + 'static,
        {
            let other = other.clone();

            self.listen(move |msg: &M| {
                other.send(msg.clone());
            });
        }

        cfg_futures!(
            pub fn channel<M>(&self) -> futures::channel::mpsc::UnboundedReceiver<M>
            where
                O: crate::Signal<M>,
                M: Clone + 'static,
            {
                let (tx, rx) = futures::channel::mpsc::unbounded();
                self.listen(move |msg: &M| {
                    tx.unbounded_send(msg.clone()).unwrap();
                });
                rx
            }
        );


        pub fn emit<M>(&self, msg: M)
        where
            O: crate::Signal<M> + 'static,
            M: 'static,
        {
            let key = self.key;
            crate::Runtime::current().inner.borrow_mut().channel.send(
                crate::rt::RuntimeMessage::Emit {
                    key,
                    msg: Box::new(msg),
                    f: Box::new(|object, key, msg| {
                        let cx = Context::<O>::new(key);
                        let object = object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.emit(cx,msg.downcast_ref().unwrap());
                    })
                },
            );
        }

        pub fn signal<M: 'static>(&self) -> crate::SignalHandle<M>
        where
            O: crate::Signal<M>  + 'static
        {
            let key = self.key;
            crate::SignalHandle {
                make_emit: alloc::rc::Rc::new(|| {
                     Box::new(|object, key, msg| {
                        let cx = Context::<O>::new(key);
                        let object = object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.emit(cx,msg.downcast_ref().unwrap());
                    })
                }),
                key: key,
                _marker: PhantomData,
            }
        }

        pub fn slot<M>(&self) -> crate::SlotHandle<M>
        where
            O: crate::Slot<M> + 'static,
            M: 'static,
        {
            let key = self.key;
            crate::SlotHandle {
                key,
                f: alloc::rc::Rc::new(core::cell::RefCell::new(
                    move |any_object: &mut dyn crate::rt::AnyObject, msg: Box<dyn core::any::Any>| {
                        let object = any_object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.handle(Context::new(key), *msg.downcast().unwrap());
                    },
                )),
                _marker: PhantomData,
            }
        }

        cfg_futures!(
            pub fn spawn_local<M>(&self, future: impl core::future::Future<Output = M> + 'static)
            where
                O: crate::Slot<M> + 'static,
                M: 'static
            {
                let me = self.clone();
                tokio::task::spawn_local(async move {
                    let msg = future.await;
                    me.send(msg)
                });
            }
        );
    );
}
