use crate::{use_hook, GLOBAL_CONTEXT, TASK_CONTEXT};
use slotmap::DefaultKey;
use std::{
    cell::{Ref, RefCell},
    fmt,
    marker::PhantomData,
    mem,
    ops::{Add, AddAssign},
    rc::Rc,
};

pub fn use_state<T: 'static>(make_value: impl FnOnce() -> T) -> State<T> {
    let rc = use_hook(|| {
        GLOBAL_CONTEXT
            .try_with(|cx| {
                cx.borrow_mut()
                    .values
                    .insert(Rc::new(RefCell::new(make_value())))
            })
            .unwrap()
    });
    let guard = rc.borrow();
    let key: &DefaultKey = guard.downcast_ref().unwrap();

    State {
        key: *key,
        _marker: PhantomData,
    }
}

pub struct State<T> {
    key: DefaultKey,
    _marker: PhantomData<T>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for State<T> {}

impl<T: 'static> State<T> {
    pub fn get(self) -> Ref<'static, T> {
        let rc = GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow_mut().values[self.key].clone())
            .unwrap();
        let output: Ref<'_, T> = Ref::map(rc.borrow(), |value| value.downcast_ref().unwrap());
        unsafe { mem::transmute(output) }
    }

    pub fn set(&self, value: T) {
        GLOBAL_CONTEXT
            .try_with(|cx| {
                *cx.borrow_mut().values[self.key]
                    .borrow_mut()
                    .downcast_mut()
                    .unwrap() = value
            })
            .unwrap();

        TASK_CONTEXT
            .try_with(|cx| {
                let guard = cx.borrow_mut();
                let cx = guard.as_ref().unwrap();
                let tx = cx.tx.clone();
                cx.local_set.borrow_mut().spawn_local(async move {
                    tx.send(Box::new(())).unwrap();
                });
            })
            .unwrap();
    }

    pub fn cloned(self) -> T
    where
        T: Clone,
    {
        self.get().clone()
    }
}

impl<T> fmt::Debug for State<T>
where
    T: fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl<T> AddAssign<T> for State<T>
where
    T: Add<Output = T> + Clone + 'static,
{
    fn add_assign(&mut self, rhs: T) {
        self.set(self.cloned() + rhs)
    }
}
