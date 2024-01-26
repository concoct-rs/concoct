//! Concoct is a framework for user-interfaces in Rust.
//!
//! This crate provides a virtual DOM and state management system for any backend.
//! Concoct uses static typing to describe your UI at compile-time to create an efficient
//! tree without allocations.
//!
//! ```ignore
//! use concoct::{Scope, View};
//! use concoct_web::html;
//! 
//! #[derive(Default)]
//! struct Counter {
//!     count: i32,
//! }
//! 
//! impl View<Self> for Counter {
//!     fn body(&mut self, _cx: &Scope<Self>) -> impl View<Self> {
//!         (
//!             format!("High five count: {}", self.count),
//!             html::button("Up high!").on_click(|_cx, state: &mut Self, _event| state.count += 1),
//!             html::button("Down low!").on_click(|_cx, state: &mut Self, _event| state.count -= 1),
//!         )
//!     }
//! }
//! 
//! fn main() {
//!     concoct_web::launch(Counter::default())
//! }
//! ```
//! 

#![deny(missing_docs)]

use std::ops::DerefMut;

mod action;
pub use self::action::{Action, IntoAction};

mod handle;
pub use self::handle::Handle;

pub mod hook;

mod vdom;
pub use self::vdom::VirtualDom;

pub mod view;
pub use self::view::View;

mod scope;
pub use self::scope::Scope;

/// Run a view on a new virtual dom.
pub async fn run<T, V>(content: V)
where
    T: 'static,
    V: View<T> + DerefMut<Target = T>,
{
    let mut vdom = VirtualDom::new(content);
    vdom.build();

    loop {
        vdom.rebuild().await;
    }
}
