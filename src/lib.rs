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
//! They can be also be ran as static methods for your object, such as `MyObject::my_slot(&mut object.cx(), my_data)`.
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
//! fn main() {
//!     let a = Counter::default().start();
//!     let b = Counter::default().start();
//!
//!     a.bind(&b, Counter::set_value);
//!
//!     Counter::set_value(&mut a.cx(), 2);
//!
//!     assert_eq!(a.borrow().value, 2);
//!     assert_eq!(b.borrow().value, 2);
//! }
//! ```
#![no_std]
#![deny(missing_docs)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Weak;
use alloc::vec::Vec;
use core::any::{Any, TypeId};
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;

mod context;
pub use self::context::Context;

mod handle;
pub use self::handle::{Binding, Handle};

pub mod signal;
pub use self::signal::Signal;

/// A reactive object.
pub trait Object {
    /// Called when the object is first started.
    #[allow(unused_variables)]
    fn started(cx: &mut Context<Self>)
    where
        Self: Sized + 'static,
    {
    }

    /// Start this object and create a handle.
    fn start(self) -> Handle<Self>
    where
        Self: Sized + 'static,
    {
        let handle = Handle::new(self);
        Self::started(&mut handle.cx());
        handle
    }
}

impl<O: Object + ?Sized> Object for &mut O {}

impl<F> Object for Box<F> where F: Object + ?Sized {}

impl<P> Object for Pin<P>
where
    P: DerefMut,
    <P as Deref>::Target: Object,
{
}

#[derive(Clone)]
struct ListenerData {
    msg_id: TypeId,
    listener_id: TypeId,
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
