use alloc::rc::Rc;
use core::{
    cell::{self, RefCell},
    marker::PhantomData,
    ops::Deref,
};
use slotmap::DefaultKey;

pub(crate) struct Inner {
    #[allow(dead_code)]
    pub(crate) key: DefaultKey,
}

/// Type-erased handle to an object.
/// 
/// Dropping this handle will also despawn the attached object.
#[derive(Clone)]
pub struct HandleGuard {
    #[allow(dead_code)]
    pub(crate) inner: Rc<Inner>,
}

impl Drop for HandleGuard {
    fn drop(&mut self) {
        #[cfg(feature = "rt")]
        if let Some(rt) = crate::Runtime::try_current() {
            rt.inner.borrow_mut().objects.remove(self.inner.key);
        }
    }
}

/// Handle to a spawned object.
///
/// Dropping this handle will also despawn the attached object.
pub struct Handle<O: ?Sized> {
    pub(crate) guard: HandleGuard,
    pub(crate) _marker: PhantomData<O>,
}

impl<O> Clone for Handle<O> {
    fn clone(&self) -> Self {
        Self {
            guard: self.guard.clone(),
            _marker: PhantomData,
        }
    }
}

impl<O> Handle<O> {
    cfg_rt!(
        pub(crate) fn new(key: DefaultKey) -> Self {
            Handle {
                guard: HandleGuard {
                    inner: Rc::new(Inner { key }),
                },
                _marker: PhantomData,
            }
        }

        /// Send a message to this object.
        pub fn send<M>(&self, msg: M)
        where
            O: crate::Slot<M> + 'static,
            M: 'static,
        {
            let key = self.guard.inner.key;
            let me = self.clone();
            crate::Runtime::current().inner.borrow_mut().channel.send(
                crate::rt::RuntimeMessage::Handle {
                    key,
                    f: Box::new(move |any_object| {
                        let object = any_object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.handle(me, msg);
                    }),
                },
            )
        }

        /// Listen to messages emitted by this object.
        pub fn listen<M>(&self, mut on_message: impl FnMut(&M) + 'static)
        where
            O: crate::Signal<M>,
            M: 'static,
        {
            crate::Runtime::current()
                .inner
                .borrow_mut()
                .listeners
                .insert(
                    (self.guard.inner.key, core::any::TypeId::of::<M>()),
                    vec![alloc::rc::Rc::new(core::cell::RefCell::new(
                        move |msg: &dyn core::any::Any| on_message(msg.downcast_ref().unwrap()),
                    ))],
                );
        }

        /// Bind another object to messages emitted by this object.
        pub fn bind<M>(&self, other: &Handle<impl crate::Object + crate::Slot<M> + 'static>)
        where
            O: crate::Signal<M>,
            M: Clone + 'static,
        {
            let other = other.clone();

            self.listen(move |msg: &M| {
                other.send(msg.clone());
            });
        }

        /// Emit a message from this object.
        pub fn emit<M>(&self, msg: M)
        where
            O: crate::Signal<M> + 'static,
            M: 'static,
        {
            let key = self.guard.inner.key;
            let me = self.clone();
            crate::Runtime::current().inner.borrow_mut().channel.send(
                crate::rt::RuntimeMessage::Emit {
                    key,
                    msg: Box::new(msg),
                    f: Box::new(|object, _key, msg| {
                        let object = object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.emit(me, msg.downcast_ref().unwrap());
                    }),
                },
            );
        }


        cfg_futures!(
            /// Create a channel to messages emitted by this object.
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

        /// Borrow a reference to this object.
        pub fn borrow(&self) -> Ref<O> {
            let rc = crate::Runtime::current().inner.borrow_mut().objects[self.guard.inner.key].clone();
            let object: cell::Ref<O> = cell::Ref::map(rc.borrow(), |object| {
                object.as_any().downcast_ref().unwrap()
            });
            let object = unsafe { std::mem::transmute(object) };
            Ref {
                object: object,
                _guard: rc,
            }
        }

        /// Get a handle to this object's signal for a specific message.
        pub fn signal<M: 'static>(&self) -> crate::SignalHandle<M>
        where
            O: crate::Signal<M> + 'static,
        {
            let key = self.guard.inner.key;
            let me = self.clone();
            crate::SignalHandle {
                make_emit: alloc::rc::Rc::new(move || {
                    let me = me.clone();
                    Box::new(move |object, _key, msg| {
                        let object = object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.emit(me.clone(), msg.downcast_ref().unwrap());
                    })
                }),
                key: key,
                _marker: PhantomData,
            }
        }

        /// Get a handle to this object's slot for a specific message.
        pub fn slot<M>(&self) -> crate::SlotHandle<M>
        where
            O: crate::Slot<M> + 'static,
            M: 'static,
        {
            let key = self.guard.inner.key;
            let me = self.clone();
            crate::SlotHandle {
                key,
                f: alloc::rc::Rc::new(core::cell::RefCell::new(
                    move |any_object: &mut dyn crate::AnyObject, msg: Box<dyn core::any::Any>| {
                        let object = any_object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.handle(me.clone(), *msg.downcast().unwrap());
                    },
                )),
                _marker: PhantomData,
            }
        }

        cfg_futures!(
            /// Spawn a `!Send` future attached to this object.
            ///
            /// The output of this future will be sent to this object as a message.
            pub fn spawn_local<M>(&self, future: impl core::future::Future<Output = M> + 'static)
            where
                O: crate::Slot<M> + 'static,
                M: 'static,
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

pub struct Ref<O: 'static> {
    object: cell::Ref<'static, O>,
    _guard: Rc<RefCell<dyn crate::AnyObject>>,
}

impl<T: 'static> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}
