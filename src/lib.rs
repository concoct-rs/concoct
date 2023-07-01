use std::{
    any::TypeId,
    hash::{Hash, Hasher},
};

extern crate self as concoct;

pub use concoct_macros::composable;

mod composer;
pub use composer::Composer;
use slot_table::Slot;

pub mod slot_table;

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
pub fn remember<T: 'static, F: FnOnce() -> T + 'static>(f: F) -> T {
    composer.cache(false, f)
}

pub struct Key {
    slot: Box<dyn Slot>,
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let addr = (&*self.slot) as *const dyn Slot;
        addr.hash(state);

        self.slot.dyn_hash(state);
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.slot.any_eq(other)
    }
}

impl Eq for Key {}
