mod handle;
pub use self::handle::Handle;

mod context;
pub use self::context::Context;

mod object;
pub use self::object::Object;

mod rt;
pub use self::rt::{Runtime, RuntimeGuard};

mod slot_handle;
pub use slot_handle::SlotHandle;

mod signal_handle;
pub use signal_handle::SignalHandle;

pub trait Signal<M>: Object {}

pub trait Slot<M>: Object {
    fn handle(&mut self, handle: Context<Self>, msg: M);
}
