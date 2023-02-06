use crate::{Composer, Modifier};
use std::{
    any::{Any, TypeId},
    rc::Rc,
};

use super::container;

pub fn context<T: 'static>() -> Option<Rc<T>> {
    Composer::with(|composer| {
        composer
            .borrow()
            .contexts
            .get(&TypeId::of::<T>())
            .map(|rc| rc.clone().downcast().unwrap())
    })
}

#[track_caller]
pub fn provide_context<T: 'static>(value: T, composable: impl FnMut() + 'static) {
    Composer::with(|composer| {
        composer
            .borrow_mut()
            .contexts
            .insert(value.type_id(), Rc::new(value))
    });

    container(Modifier, composable);

    Composer::with(|composer| {
        composer.borrow_mut().contexts.remove(&TypeId::of::<T>());
    });
}
