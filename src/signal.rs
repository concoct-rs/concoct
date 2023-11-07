use generational_box::GenerationalBox;

use crate::{STORE, SCOPE};

pub struct Signal<T> {
    value: GenerationalBox<T>,
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: SCOPE.try_with(|scope| scope.owner.insert(value)).unwrap(),
        }
    }

    pub fn read(&self) -> std::cell::Ref<'_, T>{
        self.value.read()
    }
}
