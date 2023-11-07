use crate::{Scope, STORE};
use generational_box::GenerationalBox;

pub struct Signal<T> {
    value: GenerationalBox<T>,
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Scope::current().inner.borrow_mut().owner.insert(value),
        }
    }

    pub fn read(&self) -> std::cell::Ref<'_, T> {
        self.value.read()
    }
}
