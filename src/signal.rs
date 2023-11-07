use crate::{runtime::Runtime, Scope};
use generational_box::{GenerationalBox, Owner};
use slotmap::DefaultKey;
use std::{
    collections::HashSet,
    fmt,
    ops::{AddAssign, SubAssign},
};

pub fn use_signal<T: 'static>(f: impl FnOnce() -> T) -> Signal<T> {
    let scope = Scope::current();

    let hook = scope.use_hook(|| {
        let owner = &scope.inner.borrow().owner;
        Signal::new(f(), owner)
    });
    *hook
}

pub struct Signal<T> {
    value: GenerationalBox<T>,
    key: DefaultKey,
}

impl<T: 'static> Signal<T> {
    fn new(value: T, owner: &Owner) -> Self {
        let key = Runtime::current()
            .inner
            .borrow_mut()
            .signals
            .insert(HashSet::new());

        Self {
            value: owner.insert(value),
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

impl<T: fmt::Display + 'static> fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.read().fmt(f)
    }
}

impl<T: AddAssign + 'static> AddAssign<T> for Signal<T> {
    fn add_assign(&mut self, rhs: T) {
        *self.write() += rhs
    }
}

impl<T: SubAssign + 'static> SubAssign<T> for Signal<T> {
    fn sub_assign(&mut self, rhs: T) {
        *self.write() -= rhs
    }
}
