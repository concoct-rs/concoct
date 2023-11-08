use crate::Scope;
use generational_box::GenerationalBox;
use std::{any::Any, cell::RefMut};

pub fn use_hook<T: 'static>(f: impl FnOnce() -> T) -> RefMut<'static, T> {
    let scope = Scope::current();
    let me = scope.inner.borrow_mut();
    let idx = *me.hook_idx.borrow();
    let any = if let Some(any) = me.hooks.get(idx) {
        let any = *any;
        drop(me);
        any
    } else {
        drop(me);
        let value = f();
        let mut me = scope.inner.borrow_mut();
        let any: GenerationalBox<Box<dyn Any>> = me.owner.insert(Box::new(value));
        me.hooks.push(any);
        *me.hooks.last().unwrap()
    };

    let me = scope.inner.borrow_mut();
    *me.hook_idx.borrow_mut() += 1;
    RefMut::map(any.write(), |value| value.downcast_mut().unwrap())
}
