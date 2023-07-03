//! Generic UI compiler and runtime library.
//!
//! This crate provides positional memoization where programs are defined as a composition of [`composable`] functions.
//! The results of these functions are cached based on the position of the function call.

extern crate self as concoct;

use std::any::Any;

pub use concoct_macros::composable;

pub mod snapshot;
pub use snapshot::State;

mod task;
pub use task::{spawn, Task};

mod composer;
pub use composer::Composer;

pub trait Apply {
    type NodeId: Clone;

    fn root(&mut self) -> Self::NodeId;

    fn insert(&mut self, parent_id: Self::NodeId, node: Box<dyn Any>) -> Self::NodeId;

    fn update(&mut self, node_id: Self::NodeId, node: Box<dyn Any>);
}

impl Apply for () {
    type NodeId = ();

    fn root(&mut self) -> Self::NodeId {}

    fn insert(&mut self, parent_id: Self::NodeId, node: Box<dyn Any>) -> Self::NodeId {}

    fn update(&mut self, node_id: Self::NodeId, node: Box<dyn Any>) {
        
    }
}

pub trait Composable<A, T> {
    type Output;

    fn compose(self, compose: &mut Composer<A, T>, changed: u32) -> Self::Output;
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
