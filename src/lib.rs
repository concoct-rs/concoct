#![cfg_attr(docsrs, feature(doc_cfg))]

//! A cross-platform framework for efficient user interfaces.
//!
//! Concoct is statically-typed UI library for building applications with Rust
//! that run on multiple platforms.

mod modify;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

pub use modify::Modify;

pub mod view;
pub use view::View;

#[cfg(feature = "web")]
#[cfg_attr(docsrs, doc(cfg(feature = "web")))]
pub mod web;

/// Backend rendering platform.
pub trait Platform {
    type Event;

    /// Advance the element count.
    /// This should be called when a view is skipped.
    fn advance(&mut self);
}

#[derive(Clone, Default)]
pub struct State<T> {
    value: Rc<T>,
}

impl<T> State<T> {
    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        Rc::make_mut(&mut self.value)
    }
}

impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

impl<T, U> AsRef<U> for State<T>
where
    T: AsRef<U>,
    U: ?Sized,
{
    fn as_ref(&self) -> &U {
        self.value.as_ref().as_ref()
    }
}
