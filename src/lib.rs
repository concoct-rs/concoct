#![cfg_attr(docsrs, feature(doc_cfg))]

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

mod context;
pub use self::context::Context;

mod object;
pub use self::object::Object;

cfg_rt!(
    mod handle;
    pub use self::handle::Handle;

    pub mod rt;
    pub use self::rt::Runtime;

    mod slot_handle;
    pub use slot_handle::SlotHandle;

    mod signal_handle;
    pub use signal_handle::SignalHandle;
);

pub trait Signal<M>: Object {}

pub trait Slot<M>: Object {
    fn handle(&mut self, handle: Context<Self>, msg: M);
}
