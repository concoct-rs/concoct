use crate::{LocalContext, GLOBAL_CONTEXT};
use slotmap::DefaultKey;
use std::{
    any::Any,
    cell::{Ref, RefCell},
    marker::PhantomData,
    mem,
    rc::Rc,
};

/// A hook that lets you reference a value that’s not needed for rendering.
pub fn use_ref<T: 'static>(make_value: impl FnOnce() -> T) -> UseRef<T> {
    let rc = use_hook_value(|| {
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

    UseRef {
        key: *key,
        _marker: PhantomData,
    }
}

pub fn use_hook_value<T: 'static>(make_value: impl FnOnce() -> T) -> Rc<RefCell<dyn Any>> {
    let cx = LocalContext::current();
    let mut inner = cx.inner.borrow_mut();
    let mut hooks = inner.hooks.borrow_mut();

    let value = if let Some(hook) = hooks.get(inner.idx) {
        let value = hook.clone();

        value
    } else {
        hooks.push(Rc::new(RefCell::new(make_value())));
        hooks.last().as_deref().unwrap().clone()
    };

    drop(hooks);
    inner.idx += 1;

    value
}

pub struct UseRef<T> {
    pub key: DefaultKey,
    _marker: PhantomData<T>,
}

impl<T> Clone for UseRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UseRef<T> {}

impl<T: 'static> UseRef<T> {
    pub fn get(self) -> Ref<'static, T> {
        let rc = GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow_mut().values[self.key].clone())
            .unwrap();
        let output: Ref<'_, T> = Ref::map(rc.borrow(), |value| value.downcast_ref().unwrap());
        unsafe { mem::transmute(output) }
    }

    pub fn cloned(self) -> T
    where
        T: Clone,
    {
        self.get().clone()
    }
}
