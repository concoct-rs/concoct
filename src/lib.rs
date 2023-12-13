//! # Concoct
//!
//! Concoct is a runtime for user-interfaces in Rust.
//!
//! ## Feature flags
//! Concoct uses a set of feature flags to provide support for `#![no_std]`
//! (and to reduce the amount of compiled code).
//!
//!  - `full`: Enables all features listed below.
//!  - `rt`: Enables the `Runtime`.
//!  - `futures`: Enables interop with the `futures` crate (and provides the default `Runtime`).
//!
//! ```
//! use concoct::{Handle, Object, Runtime, Signal, Slot};
//!
//! #[derive(Default)]
//! pub struct Counter {
//!     value: i32,
//! }
//!
//! impl Object for Counter {}
//!
//! impl Signal<i32> for Counter {}
//!
//! impl Slot<i32> for Counter {
//!     fn handle(&mut self, cx: Handle<Self>, msg: i32) {
//!         if self.value != msg {
//!             self.value = msg;
//!             cx.emit(msg);
//!         }
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let rt = Runtime::default();
//!     let _guard = rt.enter();
//!
//!     let a = Counter::default().start();
//!     let b = Counter::default().start();
//!
//!     a.bind(&b);
//!
//!     a.send(1);
//!     a.send(2);
//!
//!     rt.run().await;
//!
//!     assert_eq!(a.borrow().value, 2);
//!     assert_eq!(b.borrow().value, 2);
//! }
//! ```
//!

#![cfg_attr(docsrs, feature(doc_cfg))]

#[allow(unused_macros)]
macro_rules! cfg_tokio {
    ($($i:item)*) => {
        $(
            #[cfg(feature = "tokio")]
            #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
            $i
        )*
    };
}

mod object;
pub use self::object::Object;

mod handle;
pub use self::handle::{Handle, HandleGuard};

pub mod rt;
pub use self::rt::Runtime;

mod slot_handle;
pub use slot_handle::SlotHandle;

mod signal_handle;
pub use signal_handle::SignalHandle;

/// Signal emitter of messages for an object.
pub trait Signal<M>: Object {
    /// Called when a message is emitted.
    #[allow(unused_variables)]
    fn emit(&mut self, cx: Handle<Self>, msg: &M) {}

    /// Called when a listener starts on this signal.
    #[allow(unused_variables)]
    fn listen(&mut self, cx: Handle<Self>) {}
}

/// Slot handler of messages for an object.
pub trait Slot<M>: Object {
    /// Handle a message.
    fn handle(&mut self, cx: Handle<Self>, msg: M);
}
