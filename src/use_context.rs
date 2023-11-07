use crate::Scope;
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::Deref,
    rc::Rc,
};

pub fn use_context_provider<T: 'static>(make_value: impl FnOnce() -> T) -> UseContext<T> {
    Scope::current()
        .use_hook(|| {
            let scope = Scope::current();
            let mut inner = scope.inner.borrow_mut();
            let type_id = TypeId::of::<T>();
          
            inner.contexts.insert(type_id, Rc::new(make_value()));
            UseContext {
                value: inner.contexts.get(&type_id).unwrap().clone(),
                _marker: PhantomData::<T>,
            }
        })
        .clone()
}

pub fn use_context<T: 'static>() -> Option<UseContext<T>> {
    Scope::current()
        .use_hook(|| {
            let scope = Scope::current();
            let inner = scope.inner.borrow_mut();
            inner.contexts.get(&TypeId::of::<T>()).map(|value| {
                UseContext {
                    value: value.clone(),
                    _marker: PhantomData::<T>,
                }
            })
           
        })
        .clone()
}

pub struct UseContext<T> {
    value: Rc<dyn Any>,
    _marker: PhantomData<T>,
}

impl<T: 'static> Deref for UseContext<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.downcast_ref().unwrap()
    }
}

impl<T> Clone for UseContext<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _marker: self._marker.clone(),
        }
    }
}
