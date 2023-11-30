use crate::{LocalContext, GLOBAL_CONTEXT};
use slotmap::DefaultKey;
use std::{
    any::Any,
    cell::RefCell,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// A hook that lets you reference a value that’s not needed for rendering.
pub fn use_ref<T: 'static>(make_value: impl FnOnce() -> T) -> UseRef<T> {
    let rc = use_hook_value(|| {
        let value = make_value();
        GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow_mut().values.insert(Rc::new(RefCell::new(value))))
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
    let inner = cx.scope.borrow_mut();
    let hooks = inner.hooks.borrow();

    let value = if let Some(hook) = hooks.get(inner.idx) {
        let hook = hook.clone();
        drop(hooks);
        drop(inner);
        hook
    } else {
        drop(hooks);
        drop(inner);

        let value = make_value();

        let cx = LocalContext::current();
        let inner = cx.scope.borrow_mut();
        let mut hooks = inner.hooks.borrow_mut();

        hooks.push(Rc::new(RefCell::new(value)));
        hooks.last().unwrap().clone()
    };

    let cx = LocalContext::current();
    let mut inner = cx.scope.borrow_mut();
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
    pub fn get(self) -> Ref<T> {
        let rc = GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow().values[self.key].clone())
            .unwrap();
        let value = std::cell::Ref::map(rc.borrow(), |cell| cell.downcast_ref::<T>().unwrap());
        let value = unsafe { mem::transmute(value) };

        Ref { _rc: rc, value }
    }

    pub fn get_mut(self) -> RefMut<T> {
        let rc = GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow().values[self.key].clone())
            .unwrap();
        let value =
            std::cell::RefMut::map(rc.borrow_mut(), |cell| cell.downcast_mut::<T>().unwrap());
        let value = unsafe { mem::transmute(value) };

        RefMut { _rc: rc, value }
    }
}

pub struct Ref<T: 'static> {
    pub(crate) value: std::cell::Ref<'static, T>,
    pub(crate) _rc: Rc<RefCell<dyn Any>>,
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct RefMut<T: 'static> {
    value: std::cell::RefMut<'static, T>,
    _rc: Rc<RefCell<dyn Any>>,
}

impl<T> Deref for RefMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for RefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
