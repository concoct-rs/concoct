use crate::{rt::AnyTask, Context, Handler, Object, Runtime, Signal, SignalHandle};
use futures::channel::mpsc;
use slotmap::DefaultKey;
use std::{
    cell::{self, RefCell},
    marker::PhantomData,
    mem,
    ops::Deref,
    rc::Rc,
};

struct Dropper {
    key: DefaultKey,
}

impl Drop for Dropper {
    fn drop(&mut self) {
        if let Some(rt) = Runtime::try_current() {
            rt.inner.borrow_mut().tasks.remove(self.key);
        }
    }
}

pub struct Handle<T: ?Sized> {
    dropper: Rc<Dropper>,
    _marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            dropper: self.dropper.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<T> Handle<T> {
    pub(crate) fn new(key: DefaultKey) -> Self {
        Handle {
            dropper: Rc::new(Dropper { key }),
            _marker: PhantomData,
        }
    }

    pub fn send<M>(&self, msg: M)
    where
        T: Handler<M> + 'static,
        M: 'static,
    {
        Context::<T>::new(self.dropper.key).send(msg)
    }

    pub fn listen<M>(&self, f: impl FnMut(&M) + 'static)
    where
        M: 'static,
        T: Signal<M>,
    {
        Context::<T>::new(self.dropper.key).listen(f)
    }

    pub fn bind<M, T2>(&self, other: &Handle<T2>)
    where
        T: Signal<M>,
        M: Clone + 'static,
        T2: Object + Handler<M> + 'static,
    {
        Context::<T>::new(self.dropper.key).bind(&Context::<T2>::new(other.dropper.key))
    }

    pub fn channel<M>(&self) -> mpsc::UnboundedReceiver<M>
    where
        T: Signal<M>,
        M: Clone + 'static,
    {
        Context::<T>::new(self.dropper.key).channel()
    }

    pub fn borrow(&self) -> Ref<T> {
        let rc = Runtime::current().inner.borrow_mut().tasks[self.dropper.key].clone();
        let task: cell::Ref<T> =
            cell::Ref::map(rc.borrow(), |task| task.as_any().downcast_ref().unwrap());
        let task = unsafe { mem::transmute(task) };
        Ref { task, _guard: rc }
    }

    pub fn signal<M>(&self) -> SignalHandle<M>
    where
        T: Signal<M>,
    {
        Context::<T>::new(self.dropper.key).signal()
    }
}

pub struct Ref<T: 'static> {
    task: cell::Ref<'static, T>,
    _guard: Rc<RefCell<dyn AnyTask>>,
}

impl<T: 'static> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.task
    }
}
