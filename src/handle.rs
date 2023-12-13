use slotmap::DefaultKey;
use std::{
    cell::{self, RefCell},
    marker::PhantomData,
    ops::Deref,
    rc::Rc,
};

use crate::{Runtime, Signal, Slot};

/// Handle to a spawned object.
///
/// Dropping this handle will also despawn the attached object.
///
/// ## Creating a `Handle`
/// ```rust
/// # let rt = concoct::Runtime::default();
/// # let _guard = rt.enter();
/// use concoct::Object;
///
/// struct Example;
///
/// impl Object for Example {}
///
/// let handle = Example.start();
/// ```
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
        O: Slot<M> + 'static,
        M: 'static,
    {
        let key = self.guard.inner.key;
        let me = self.clone();
        Runtime::current()
            .tx
            .unbounded_send(crate::rt::RuntimeMessage(
                crate::rt::RuntimeMessageKind::Handle {
                    key,
                    f: Box::new(move |any_object| {
                        let object = any_object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.handle(me, msg);
                    }),
                },
            ))
            .unwrap();
    }

    /// Listen to messages emitted by this object.
    pub fn listen<M>(&self, mut on_message: impl FnMut(&M) + 'static)
    where
        O: Signal<M> + 'static,
        M: 'static,
    {
        let rt = Runtime::current().inner;
        let mut rt = rt.borrow_mut();

        rt.listeners.insert(
            (self.guard.inner.key, std::any::TypeId::of::<M>()),
            vec![std::rc::Rc::new(std::cell::RefCell::new(
                move |msg: &dyn std::any::Any| on_message(msg.downcast_ref().unwrap()),
            ))],
        );

        let object = rt.objects[self.guard.inner.key].clone();
        drop(rt);

        let cx = self.clone();
        object
            .borrow_mut()
            .as_any_mut()
            .downcast_mut::<O>()
            .unwrap()
            .listen(cx);
    }

    /// Bind another object to messages emitted by this object.
    pub fn bind<M>(&self, other: &Handle<impl crate::Object + Slot<M> + 'static>)
    where
        O: Signal<M> + 'static,
        M: Clone + 'static,
    {
        let other = other.clone();

        self.listen(move |msg: &M| {
            other.send(msg.clone());
        });
    }

    /// Bind another object to messages emitted by this object.
    pub fn map<M, M2>(
        &self,
        other: &Handle<impl crate::Object + Slot<M2> + 'static>,
        mut f: impl FnMut(&M) -> M2 + 'static,
    ) where
        O: Signal<M> + 'static,
        M: 'static,
        M2: 'static,
    {
        let other = other.clone();

        self.listen(move |msg: &M| {
            other.send(f(msg));
        });
    }

    /// Emit a message from this object.
    pub fn emit<M>(&self, msg: M)
    where
        O: Signal<M> + 'static,
        M: 'static,
    {
        let key = self.guard.inner.key;
        let me = self.clone();
        Runtime::current()
            .tx
            .unbounded_send(crate::rt::RuntimeMessage(
                crate::rt::RuntimeMessageKind::Emit {
                    key,
                    msg: Box::new(msg),
                    f: Box::new(|object, _key, msg| {
                        let object = object.as_any_mut().downcast_mut::<O>().unwrap();
                        object.emit(me, msg.downcast_ref().unwrap());
                    }),
                },
            ))
            .unwrap();
    }

    /// Create a channel to messages emitted by this object.
    pub fn channel<M>(&self) -> futures::channel::mpsc::UnboundedReceiver<M>
    where
        O: Signal<M> + 'static,
        M: Clone + 'static,
    {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.listen(move |msg: &M| {
            tx.unbounded_send(msg.clone()).unwrap();
        });
        rx
    }

    /// Borrow a reference to this object.
    pub fn borrow(&self) -> Ref<O> {
        let rc = Runtime::current().inner.borrow_mut().objects[self.guard.inner.key].clone();
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
        O: Signal<M> + 'static,
    {
        let key = self.guard.inner.key;
        let me = self.clone();
        crate::SignalHandle {
            make_emit: std::rc::Rc::new(move || {
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
        O: Slot<M> + 'static,
        M: 'static,
    {
        let key = self.guard.inner.key;
        let me = self.clone();
        crate::SlotHandle {
            key,
            f: std::rc::Rc::new(std::cell::RefCell::new(
                move |any_object: &mut dyn crate::object::AnyObject,
                      msg: Box<dyn std::any::Any>| {
                    let object = any_object.as_any_mut().downcast_mut::<O>().unwrap();
                    object.handle(me.clone(), *msg.downcast().unwrap());
                },
            )),
            _marker: PhantomData,
        }
    }

    cfg_tokio!(
        /// Spawn a `!Send` future attached to this object.
        ///
        /// The output of this future will be sent to this object as a message.
        ///
        /// ```
        /// # let rt = concoct::Runtime::default();
        /// # let _guard = rt.enter();
        /// # let tokio_rt  = tokio::runtime::Runtime::new().unwrap();
        /// # tokio::task::LocalSet::new().block_on(&tokio_rt, async {
        /// use concoct::{Handle, Object, Slot};
        ///
        /// struct Example;
        ///
        /// impl Object for Example {}
        ///
        /// impl Slot<i32> for Example {
        ///     fn handle(&mut self, _cx: Handle<Self>, msg: i32) {
        ///         assert_eq!(msg, 1);
        ///     }
        /// }
        ///
        /// Example.start().spawn_local(async move { 1 });
        /// # rt.run().await;
        /// # })
        /// ```
        pub fn spawn_local<M>(&self, future: impl std::future::Future<Output = M> + 'static)
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
}

/// Type-erased handle to an object.
///
/// Dropping this handle will also despawn the attached object.
#[derive(Clone)]
pub struct HandleGuard {
    pub(crate) inner: Rc<Inner>,
}

pub(crate) struct Inner {
    pub(crate) key: DefaultKey,
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(rt) = Runtime::try_current() {
            rt.inner.borrow_mut().objects.remove(self.key);
        }
    }
}

pub struct Ref<O: 'static> {
    object: cell::Ref<'static, O>,
    _guard: Rc<RefCell<dyn crate::object::AnyObject>>,
}

impl<T: 'static> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}
