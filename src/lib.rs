//! Generic UI compiler and runtime library.
//!
//! This crate provides positional memoization where programs are defined as a composition of [`composable`] functions.
//! The results of these functions are cached based on the position of the function call.

extern crate self as concoct;

pub mod snapshot;
pub use snapshot::State;

mod task;
pub use task::{spawn, Task};

pub use concoct_macros::composable;

mod composer;
#[doc(hidden)]
pub use composer::Composer;

pub trait Composable<T, U> {
    type Output;

    fn compose(self, compose: &mut Composer<T, U>, changed: u32) -> Self::Output;
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
    T: Clone + 'static,
    F: FnOnce() -> T + 'static,
{
    composer.cache(false, f)
}

pub enum Operation<T, U> {
    Insert { parent_id: T, node: U },
}
