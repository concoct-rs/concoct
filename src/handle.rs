use crate::{rt::AnyObject, Context, Object, Runtime, Signal, SignalHandle, Slot};
use alloc::rc::Rc;
use core::{
    cell::{self, RefCell},
    marker::PhantomData,
    mem,
    ops::Deref,
};
use slotmap::DefaultKey;

pub(crate) struct Dropper {
    pub(crate) key: DefaultKey,
}

impl Drop for Dropper {
    fn drop(&mut self) {
        if let Some(rt) = Runtime::try_current() {
            rt.inner.borrow_mut().objects.remove(self.key);
        }
    }
}

pub struct Handle<O: ?Sized> {
    pub(crate) dropper: Rc<Dropper>,
    _marker: PhantomData<O>,
}

impl<O> Clone for Handle<O> {
    fn clone(&self) -> Self {
        Self {
            dropper: self.dropper.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<O> Handle<O> {
    pub(crate) fn new(key: DefaultKey) -> Self {
        Handle {
            dropper: Rc::new(Dropper { key }),
            _marker: PhantomData,
        }
    }

    pub fn send<M>(&self, msg: M)
    where
        O: Slot<M> + 'static,
        M: 'static,
    {
        Context::<O>::new(self.dropper.key).send(msg)
    }

    pub fn listen<M>(&self, f: impl FnMut(&M) + 'static)
    where
        M: 'static,
        O: Signal<M>,
    {
        Context::<O>::new(self.dropper.key).listen(f)
    }

    pub fn bind<M>(&self, other: &Handle<impl Object + Slot<M> + 'static>)
    where
        O: Signal<M>,
        M: Clone + 'static,
    {
        Context::<O>::new(self.dropper.key).bind(&Context::from_handle(other))
    }

    cfg_futures!(
        pub fn channel<M>(&self) -> futures::channel::mpsc::UnboundedReceiver<M>
        where
            O: Signal<M>,
            M: Clone + 'static,
        {
            Context::<O>::new(self.dropper.key).channel()
        }
    );

    pub fn borrow(&self) -> Ref<O> {
        let rc = Runtime::current().inner.borrow_mut().objects[self.dropper.key].clone();
        let object: cell::Ref<O> = cell::Ref::map(rc.borrow(), |object| {
            object.as_any().downcast_ref().unwrap()
        });
        let object = unsafe { mem::transmute(object) };
        Ref {
            object: object,
            _guard: rc,
        }
    }

    pub fn signal<M>(&self) -> SignalHandle<M>
    where
        O: Signal<M> + 'static,
        M: 'static,
    {
        Context::<O>::new(self.dropper.key).signal()
    }

    cfg_futures!(
        pub fn spawn_local<M>(&self, future: impl core::future::Future<Output = M> + 'static)
        where
            O: crate::Slot<M> + 'static,
            M: 'static,
        {
            Context::<O>::new(self.dropper.key).spawn_local(future)
        }
    );
}

pub struct Ref<O: 'static> {
    object: cell::Ref<'static, O>,
    _guard: Rc<RefCell<dyn AnyObject>>,
}

impl<T: 'static> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.object
    }
}
