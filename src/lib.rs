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
    fn root(&mut self) -> Box<dyn Any>;

    fn insert(&mut self, parent_id: &dyn Any, node: Box<dyn Any>) -> Box<dyn Any>;

    fn update(&mut self, node_id: &dyn Any, node: Box<dyn Any>);

    fn remove(&mut self, node_id: &dyn Any);
}

impl Apply for () {
    fn root(&mut self) -> Box<dyn Any> {
        Box::new(())
    }

    fn insert(&mut self, _parent_id: &dyn Any, _node: Box<dyn Any>) -> Box<dyn Any> {
        Box::new(())
    }

    fn update(&mut self, _node_id: &dyn Any, _node: Box<dyn Any>) {}

    fn remove(&mut self, _node_id: &dyn Any) {}
}

pub trait Composable {
    type Output;

    fn compose(self, compose: &mut Composer, changed: u32) -> Self::Output;
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

#[composable]
pub fn node<T>(node: T)
where
    T: Clone + Send + 'static,
{
    composer.node(Box::new(node.clone()))
}

#[composable]
pub fn provide<T>(context: T)
where
    T: Clone + Send + 'static,
{
    composer.provide(Box::new(context.clone()))
}

#[composable]
pub fn context<T>() -> T
where
    T: Clone + 'static,
{
    composer.context()
}
