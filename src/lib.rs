//! A high-performance runtime for reactive objects.
//!
//! Concoct is an event-driven state management system that run anywhere (including `#![no_std]`).
//!
//! The main component is an `Object`, a reactive group of data,
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
//! They can be also be ran as static methods for your object, such as `MyObject::my_slot(&mut object.cx(), my_data)`.
//!
#![no_std]
#![deny(missing_docs)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::any::{Any, TypeId};
use core::cell::RefCell;

mod context;
pub use self::context::Context;

mod handle;
pub use self::handle::{Handle, Listener};

pub mod signal;
pub use self::signal::Signal;

/// A reactive object.
pub trait Object {
    /// Start this object and create a handle.
    fn start(self) -> Handle<Self>
    where
        Self: Sized + 'static,
    {
        Handle::new(self)
    }
}

#[derive(Clone)]
struct ListenerData {
    msg_id: TypeId,
    listener_id: TypeId,
    f: Rc<RefCell<dyn FnMut(&dyn Any)>>,
}

struct Node {
    object: Box<dyn Any>,
    listeners: Vec<ListenerData>,
}
