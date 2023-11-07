use crate::{runtime::Runtime, Scope};
use generational_box::GenerationalBox;
use slotmap::DefaultKey;

pub struct Signal<T> {
    value: GenerationalBox<T>,
    scope_key: DefaultKey,
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let scope = Scope::current();
        let inner = scope.inner.borrow_mut();
        Self {
            value: inner.owner.insert(value),
            scope_key: inner.key,
        }
    }

    pub fn read(&self) -> std::cell::Ref<'_, T> {
        self.value.read()
    }

    pub fn write(&self) -> std::cell::RefMut<'_, T> {
        Runtime::current().update(self.scope_key);
        self.value.write()
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Signal<T> {}
