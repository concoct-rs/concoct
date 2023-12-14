use crate::{
    object::AnyObject,
    rt::{Node, RuntimeMessage, RuntimeMessageKind},
    Object, Runtime, Signal, SignalHandle, Slot, SlotHandle,
};
use futures::{
    channel::mpsc::{self, UnboundedSender},
    Stream, StreamExt,
};
use slotmap::DefaultKey;
use std::{
    any::TypeId,
    cell::{self, RefCell},
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

pub(crate) static LISTENER_ID: AtomicU64 = AtomicU64::new(0);

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

impl<O> Handle<O> {
    pub(crate) fn new(key: DefaultKey, tx: UnboundedSender<RuntimeMessage>) -> Self {
        Handle {
            guard: HandleGuard {
                inner: Arc::new(Inner { key, tx }),
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
        self.guard
            .inner
            .tx
            .unbounded_send(RuntimeMessage(RuntimeMessageKind::Handle {
                key,
                f: Box::new(move |any_object| {
                    let object = any_object.as_any_mut().downcast_mut::<O>().unwrap();
                    object.update(me, msg);
                }),
            }))
            .unwrap();
    }

    /// Listen to messages emitted by this object.
    #[must_use]
    pub fn listen<M>(&self, on_message: impl FnMut(&M) + 'static) -> ListenerGuard
    where
        O: Signal<M> + 'static,
        M: 'static,
    {
        ListenerGuard {
            handle: self.listen_inner(on_message, None),
        }
    }

    /// Bind another object to messages emitted by this object.
    pub fn bind<M>(&self, other: &Handle<impl Object + Slot<M> + 'static>) -> BindHandle
    where
        O: Signal<M> + 'static,
        M: Clone + 'static,
    {
        self.map(other, |msg| msg.clone())
    }

    /// Bind another object to messages emitted by this object.
    pub fn map<M, M2>(
        &self,
        other: &Handle<impl Object + Slot<M2> + 'static>,
        mut f: impl FnMut(&M) -> M2 + 'static,
    ) -> BindHandle
    where
        O: Signal<M> + 'static,
        M: 'static,
        M2: 'static,
    {
        let other = other.clone();
        let guard = other.guard.clone();
        self.listen_inner(
            move |msg: &M| {
                other.send(f(msg));
            },
            Some(guard),
        )
    }

    /// Emit a message from this object.
    pub fn emit<M>(&self, msg: M)
    where
        O: Signal<M> + 'static,
        M: 'static,
    {
        let key = self.guard.inner.key;
        let me = self.clone();
        self.guard
            .inner
            .tx
            .unbounded_send(RuntimeMessage(RuntimeMessageKind::Emit {
                key,
                msg: Box::new(msg),
                f: Box::new(|object, _key, msg| {
                    let object = object.as_any_mut().downcast_mut::<O>().unwrap();
                    object.emit(me, msg.downcast_ref().unwrap());
                }),
            }))
            .unwrap();
    }

    /// Create a channel to messages emitted by this object.
    pub fn channel<M>(&self) -> Channel<M>
    where
        O: Signal<M> + 'static,
        M: Clone + 'static,
    {
        let (tx, rx) = mpsc::unbounded();
        let guard = self.listen_inner(
            move |msg: &M| {
                tx.unbounded_send(msg.clone()).unwrap();
            },
            None,
        );
        Channel { rx, _guard: guard }
    }

    /// Borrow a reference to this object.
    pub fn borrow(&self) -> Ref<O> {
        let rc = Runtime::current().inner.borrow_mut().nodes[self.guard.inner.key].clone();
        let object: cell::Ref<O> = cell::Ref::map(rc.borrow(), |node: &Node| {
            node.object.as_any().downcast_ref().unwrap()
        });
        let object = unsafe { std::mem::transmute(object) };
        Ref {
            object: object,
            _guard: rc,
        }
    }

    /// Get a handle to this object's signal for a specific message.
    pub fn signal<M: 'static>(&self) -> SignalHandle<M>
    where
        O: Signal<M> + 'static,
    {
        let me = self.clone();
        let cx = me.clone();

        SignalHandle {
            make_emit: Arc::new(move || {
                let me = me.clone();
                Box::new(move |object, _key, msg| {
                    let object = object.as_any_mut().downcast_mut::<O>().unwrap();
                    object.emit(me.clone(), msg.downcast_ref().unwrap());
                })
            }),
            handle: self.guard.clone(),
            make_listen: Arc::new(move || {
                let cx = cx.clone();
                Box::new(move |object| {
                    object
                        .as_any_mut()
                        .downcast_mut::<O>()
                        .unwrap()
                        .listen(cx.clone());
                })
            }),
            _marker: PhantomData,
        }
    }

    /// Get a handle to this object's slot for a specific message.
    pub fn slot<M>(&self) -> SlotHandle<M>
    where
        O: Slot<M> + 'static,
        M: 'static,
    {
        let key = self.guard.inner.key;
        let me = self.clone();
        SlotHandle {
            key,
            f: Arc::new(
                move |any_object: &mut dyn AnyObject, msg: Box<dyn std::any::Any>| {
                    let object = any_object.as_any_mut().downcast_mut::<O>().unwrap();
                    object.update(me.clone(), *msg.downcast().unwrap());
                },
            ),
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
        ///     fn update(&mut self, _cx: Handle<Self>, msg: i32) {
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

    fn listen_inner<M>(
        &self,
        mut on_message: impl FnMut(&M) + 'static,
        listener: Option<HandleGuard>,
    ) -> BindHandle
    where
        O: Signal<M> + 'static,
        M: 'static,
    {
        let id = LISTENER_ID.fetch_add(1, Ordering::SeqCst);
        let cx = self.clone();

        self.guard
            .inner
            .tx
            .unbounded_send(RuntimeMessage(RuntimeMessageKind::Listen {
                id,
                key: self.guard.inner.key,
                type_id: TypeId::of::<M>(),
                f: Rc::new(RefCell::new(move |msg: &dyn std::any::Any| {
                    on_message(msg.downcast_ref().unwrap())
                })),
                listen_f: Box::new(|object| {
                    object.as_any_mut().downcast_mut::<O>().unwrap().listen(cx);
                }),
                listener,
            }))
            .unwrap();

        BindHandle {
            id,
            type_id: TypeId::of::<M>(),
            handle: self.guard.clone(),
        }
    }
}

impl<O> Clone for Handle<O> {
    fn clone(&self) -> Self {
        Self {
            guard: self.guard.clone(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<O> Send for Handle<O> {}

unsafe impl<O> Sync for Handle<O> {}

impl<O> Unpin for Handle<O> {}

/// Type-erased handle to an object.
///
/// Dropping this handle will also despawn the attached object.
#[derive(Clone)]
pub struct HandleGuard {
    pub(crate) inner: Arc<Inner>,
}

pub(crate) struct Inner {
    pub(crate) key: DefaultKey,
    pub(crate) tx: UnboundedSender<RuntimeMessage>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.tx
            .unbounded_send(RuntimeMessage(RuntimeMessageKind::Remove { key: self.key }))
            .ok();
    }
}

pub struct Ref<O: 'static> {
    object: cell::Ref<'static, O>,
    _guard: Rc<RefCell<Node>>,
}

impl<T: 'static> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

pub struct ListenerGuard {
    pub(crate) handle: BindHandle,
}

impl Drop for ListenerGuard {
    fn drop(&mut self) {
        self.handle.release()
    }
}

pub struct BindHandle {
    pub(crate) id: u64,
    pub(crate) type_id: TypeId,
    pub(crate) handle: HandleGuard,
}

impl BindHandle {
    pub fn release(&mut self) {
        self.handle
            .inner
            .tx
            .unbounded_send(RuntimeMessage(RuntimeMessageKind::RemoveListener {
                id: self.id,
                key: self.handle.inner.key,
                type_id: self.type_id,
            }))
            .unwrap();
    }
}

/// Stream for the [`Handle::channel`] method.
pub struct Channel<M> {
    rx: mpsc::UnboundedReceiver<M>,
    _guard: BindHandle,
}

impl<M> Stream for Channel<M> {
    type Item = M;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}
