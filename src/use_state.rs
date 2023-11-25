use crate::{
    use_ref::{use_ref, UseRef},
    BUILD_CONTEXT, GLOBAL_CONTEXT, TASK_CONTEXT,
};
use std::{
    cell::Ref,
    fmt, mem,
    ops::{Add, AddAssign},
};

/// A hook that lets you add a state variable to your composable.
pub fn use_state<T: 'static>(make_value: impl FnOnce() -> T) -> UseState<T> {
    let hook = use_ref(make_value);
    UseState { hook }
}

pub struct UseState<T> {
    hook: UseRef<T>,
}

impl<T> Clone for UseState<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UseState<T> {}

impl<T: 'static> UseState<T> {
    pub fn get(self) -> Ref<'static, T> {
        BUILD_CONTEXT
            .try_with(|cx| {
                if let Some(cx) = cx.borrow_mut().as_mut() {
                    let mut cx = cx.borrow_mut();
                    let key = cx.parent_key;
                    cx.tracked.insert(key, vec![self.hook.key]);
                }
            })
            .unwrap();

        let rc = GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow_mut().values[self.hook.key].clone())
            .unwrap();
        let output: Ref<'_, T> = Ref::map(rc.borrow(), |value| value.downcast_ref().unwrap());
        unsafe { mem::transmute(output) }
    }

    pub fn set(&self, value: T) {
        GLOBAL_CONTEXT
            .try_with(|cx| {
                cx.borrow_mut().dirty.insert(self.hook.key);
                *cx.borrow_mut().values[self.hook.key]
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
                tx.unbounded_send(Box::new(())).unwrap();
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

impl<T> fmt::Debug for UseState<T>
where
    T: fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl<T> AddAssign<T> for UseState<T>
where
    T: Add<Output = T> + Clone + 'static,
{
    fn add_assign(&mut self, rhs: T) {
        self.set(self.cloned() + rhs)
    }
}
