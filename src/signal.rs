use std::collections::HashSet;

use crate::{runtime::Runtime, Scope};
use generational_box::GenerationalBox;
use slotmap::DefaultKey;

pub struct Signal<T> {
    value: GenerationalBox<T>,
    key: DefaultKey,
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let scope = Scope::current();
        let inner = scope.inner.borrow_mut();
        let key = Runtime::current()
            .inner
            .borrow_mut()
            .signals
            .insert(HashSet::new());

        Self {
            value: inner.owner.insert(value),
            key,
        }
    }

    pub fn read(&self) -> std::cell::Ref<'_, T> {
        Runtime::current().inner.borrow_mut().signals[self.key]
            .insert(Scope::current().inner.borrow().key);
        self.value.read()
    }

    pub fn write(&self) -> std::cell::RefMut<'_, T> {
        Runtime::current().update(self.key);
        self.value.write()
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Signal<T> {}
