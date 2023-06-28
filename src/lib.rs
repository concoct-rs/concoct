use std::any::TypeId;

extern crate self as concoct;

pub use concoct_macros::composable;

pub trait Compose {
    fn start_restart_group(&mut self, type_id: TypeId);

    fn end_restart_group(&mut self, f: impl FnOnce() -> Box<dyn FnMut(&mut Self)>);

    fn start_replaceable_group(&mut self, type_id: TypeId);

    fn end_replaceable_group(&mut self);

    fn is_skipping(&self) -> bool;

    fn skip_to_group_end(&mut self);

    fn cache<T>(&mut self, is_invalid: bool, f: impl FnOnce() -> T) -> T;
}

pub trait Composable {
    type Output;

    fn compose(self, compose: &mut impl Compose, changed: u32) -> Self::Output;
}

#[macro_export]
macro_rules! compose {
    ($composable:expr) => {
        $composable
    };
}

// TODO
#[macro_export]
macro_rules! current_composer {
    () => {};
}

#[composable]
pub fn remember<T: 'static>(value: T) -> i32 {
    composer.cache(false, || 0)
}
