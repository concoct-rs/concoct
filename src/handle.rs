use crate::{rt::AnyTask, Handler, Runtime, Task};
use slotmap::DefaultKey;
use std::{
    any::{Any, TypeId},
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

pub struct Handle<T> {
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
        let key = self.dropper.key;
        Runtime::current()
            .tx
            .unbounded_send((
                key,
                Box::new(move |any_task| {
                    if let Some(listeners) = Runtime::current()
                        .inner
                        .borrow()
                        .listeners
                        .get(&(key, msg.type_id()))
                        .clone()
                    {
                        for listener in listeners {
                            listener.borrow_mut()(&msg)
                        }
                    }

                    let task = any_task.as_any_mut().downcast_mut::<T>().unwrap();
                    task.handle(msg);
                }),
            ))
            .unwrap();
    }

    pub fn listen<M: 'static>(&self, mut f: impl FnMut(&M) + 'static) {
        Runtime::current().inner.borrow_mut().listeners.insert(
            (self.dropper.key, TypeId::of::<M>()),
            vec![Rc::new(RefCell::new(move |msg: &dyn Any| {
                f(msg.downcast_ref().unwrap())
            }))],
        );
    }

    pub fn bind<M, T2>(&self, other: &Handle<T2>)
    where
        M: Clone + 'static,
        T2: Task + Handler<M> + 'static,
    {
        let other = other.clone();

        self.listen(move |msg: &M| {
            other.send(msg.clone());
        });
    }

    pub fn borrow(&self) -> Ref<T> {
        let rc = Runtime::current().inner.borrow_mut().tasks[self.dropper.key].clone();
        let task: cell::Ref<T> =
            cell::Ref::map(rc.borrow(), |task| task.as_any().downcast_ref().unwrap());
        let task = unsafe { mem::transmute(task) };
        Ref { task, _guard: rc }
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
