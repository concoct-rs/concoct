//! A high-performance runtime for reactive objects.
//!
//! Concoct is an event-driven state management system that run anywhere (including `#![no_std]`).
//!
//! ## Objects
//!
//! The main component is an [`Object`], a reactive group of data,
//! that is composed of signals and slots.
//! Signals can be bound to slots to create an efficient reactive graph
//! where signals send events to slots.
//!
//! ## Signals
//! A signal is an event emitted by an object.
//! Multiple signals can be added to an object by implementing [`Signal`]
//! for the specific messages.
//!
//! ## Slots
//! A slot is a callback into the mutable state of an object.
//! As such, they can be implemented as normal functions that take a
//! [`Context`] self parameter and a message.
//!
//! ```no_run
//! use concoct::{Context, Object, Signal};
//!
//! #[derive(Default)]
//! struct Counter {
//!     value: i32,
//! }
//!
//! impl Object for Counter {}
//!
//! impl Signal<i32> for Counter {}
//!
//! impl Counter {
//!     fn set_value(cx: &mut Context<Self>, value: i32) {
//!         if cx.value != value {
//!             cx.value = value;
//!             cx.emit(value);
//!         }
//!     }
//! }
//!
//! let a = Counter::default().start();
//! let b = Counter::default().start();
//!
//! a.bind(&b, Counter::set_value);
//!
//! Counter::set_value(&mut a.cx(), 2);
//!
//! assert_eq!(a.borrow().value, 2);
//! assert_eq!(b.borrow().value, 2);
//! ```
//! 
//! ## Installation
//! The easiest way to get started is using the `full` feature flag.
//! ```ignore
//! cargo add concoct --features full
//! ```
//! 
//! ## Feature flags
//!  - `full`: Enables all the features below.
//!  - `channel`: Enables the `channel` module for channels between objects.
//! 
#![no_std]
#![deny(missing_docs)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Weak;
use alloc::vec::Vec;
use core::any::{Any, TypeId};
use core::cell::RefCell;

mod context;
pub use self::context::Context;

mod handle;
pub use self::handle::{Binding, Handle};

mod object;
pub use self::object::Object;

pub mod signal;
pub use self::signal::Signal;

#[cfg(feature = "channel")]
#[cfg_attr(docsrs, doc(cfg(feature = "channel")))]
pub mod channel;

#[derive(Clone)]
struct ListenerData {
    msg_id: TypeId,
    slot_id: TypeId,
    node: Weak<RefCell<Node>>,
    listen: fn(Weak<RefCell<Node>>, *const (), &dyn Any),
    slot: *const (),
}

struct Node {
    object: Box<dyn Any>,
    listeners: Vec<ListenerData>,
    listening: Vec<Binding>,
}

impl Drop for Node {
    fn drop(&mut self) {
        for listener in &self.listening {
            listener.unbind();
        }
    }
}
