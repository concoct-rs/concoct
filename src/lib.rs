//! Generic UI compiler and runtime library.
//!
//! This crate provides positional memoization where programs are defined as a composition of [`composable`] functions.
//! The results of these functions are cached based on the position of the function call.

use std::{any::TypeId, hash::Hash};

extern crate self as concoct;

pub mod snapshot;
pub use snapshot::{spawn, State};

pub use concoct_macros::composable;

mod composer;
#[doc(hidden)]
pub use composer::Composer;

/// Composer is the interface that is targeted by the [`composable`] macro and used by code generation helpers.
/// It is highly recommended that direct calls these be avoided as the runtime assumes that the calls are generated
/// by the compiler and contain only a minimum amount of state validation.
pub trait Compose {
    fn start_restart_group(&mut self, type_id: TypeId);

    fn end_restart_group(&mut self, f: impl FnOnce() -> Box<dyn FnOnce(&mut Self, u32)>);

    fn start_replaceable_group(&mut self, type_id: TypeId);

    fn end_replaceable_group(&mut self);

    fn is_skipping(&self) -> bool;

    fn skip_to_group_end(&mut self);

    /// Cache a value in the composition.
    /// During initial composition `f` is called to produce the value that is then stored in the slot table.
    /// During recomposition, if `is_invalid` is false the value is obtained from the slot table and `f` is not invoked.
    /// If `is_invalid` is false a new value is produced by calling [block]
    /// and the slot table is updated to contain the new value.
    fn cache<T>(&mut self, is_invalid: bool, f: impl FnOnce() -> T) -> T
    where
        T: Clone + Hash + PartialEq + 'static;

    /// Determine if the current slot table value is equal to the given value, if true, the value
    /// is scheduled to be skipped during [ControlledComposition.applyChanges] and [changes] return
    /// false; otherwise [ControlledComposition.applyChanges] will update the slot table to [value].
    /// In either case the composer's slot table is advanced.
    fn changed<T>(&mut self, value: &T) -> bool
    where
        T: Clone + Hash + PartialEq + 'static;
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

/// Remember the value produced by `f`. `f` will only be evaluated during the composition.
/// Recomposition will always return the value produced by composition.
#[composable]
pub fn remember<T, F>(f: F) -> T
where
    T: Clone + Hash + PartialEq + 'static,
    F: FnOnce() -> T + 'static,
{
    composer.cache(false, f)
}
