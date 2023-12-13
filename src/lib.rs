//! # Concoct
//!
//! ## Feature flags
//! Concoct uses a set of feature flags to provide support for `#![no_std]``
//! (and to reduce the amount of compiled code).
//!
//!  - `full`: Enables all features listed below.
//!  - `rt`: Enables the `Runtime`.
//!  - `futures`: Enables interop with the `futures` crate (and provides the default `Runtime`).
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

macro_rules! cfg_rt {
    ($($i:item)*) => {
        $(
            #[cfg(feature = "rt")]
            #[cfg_attr(docsrs, doc(cfg(feature = "rt")))]
            $i
        )*
    };
}

#[allow(unused_macros)]
macro_rules! cfg_futures {
    ($($i:item)*) => {
        $(
            #[cfg(feature = "futures")]
            #[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
            $i
        )*
    };
}

mod object;
pub use self::object::{AnyObject, Object};

mod handle;
pub use self::handle::{Handle, HandleGuard};

cfg_rt!(
    pub mod rt;
    pub use self::rt::Runtime;

    mod slot_handle;
    pub use slot_handle::SlotHandle;

    mod signal_handle;
    pub use signal_handle::SignalHandle;
);

pub trait Signal<M>: Object {
    #[allow(unused_variables)]
    fn emit(&mut self, cx: Handle<Self>, msg: &M) {}
}

pub trait Slot<M>: Object {
    fn handle(&mut self, cx: Handle<Self>, msg: M);
}
