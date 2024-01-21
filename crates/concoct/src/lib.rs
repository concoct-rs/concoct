//! A UI framework for writing declarative apps on multiple platforms.
//!
//! Concoct uses static typing to describe your UI at compile-time
//! to create an efficient tree of components. Updates to state re-render
//! your application top-down, starting at the state's parent component.
//!
//! ```ignore
//! use concoct::{View, ViewBuilder};
//! use concoct::hook::use_state;
//! use concoct_web::html;
//!
//! struct App;
//!
//! impl ViewBuilder for App {
//!     fn build(&self) -> impl View {
//!         let (count, set_high) = use_state(|| 0);
//!         let set_low = set_high.clone();
//!
//!         (
//!             format!("High five count: {}", count),
//!             html::button("Up high!").on_click(move |_| set_high(count + 1)),
//!             html::button("Down low!").on_click(move |_| set_low(count - 1)),
//!         )
//!     }
//! }
//! ```
//! 
//! ## Feature flags
//! - `full`: Enables all of the features below.
//! - `tracing`: Enables logging with the `tracing` crate.
//!

use std::borrow::Cow;
use std::cell::RefCell;

pub mod hook;

mod macros;

mod rt;
pub(crate) use self::rt::{Runtime, Scope};

mod tree;
pub(crate) use tree::Node;
pub use tree::Tree;

mod vdom;
pub use self::vdom::VirtualDom;

mod view_builder;
pub use self::view_builder::ViewBuilder;

pub mod view;
pub use self::view::View;

/// Run a view in a new virtual dom.
pub async fn run(view: impl ViewBuilder) {
    let mut vdom = VirtualDom::new(view.into_tree());
    vdom.build();

    loop {
        vdom.rebuild().await
    }
}

/// Provider for a platform-specific text view.
///
/// If you're writing a custom backend, you can use this to override
/// the default implementation of `View` for string types (like `&str` and `String`).
///
/// To expose it to child views, use [`use_provider`](`crate::hook::use_provider`).
pub struct TextViewContext {
    view: RefCell<Box<dyn FnMut(Cow<'static, str>)>>,
}

impl TextViewContext {
    /// Create a text view context from a view function.
    ///
    /// Text-based views, such as `&str` or `String` will call
    /// this view function on when rendered.
    pub fn new(view: impl FnMut(Cow<'static, str>) + 'static) -> Self {
        Self {
            view: RefCell::new(Box::new(view)),
        }
    }
}
