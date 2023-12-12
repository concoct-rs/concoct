mod handle;
pub use self::handle::Handle;

mod context;
pub use self::context::Context;

mod object;
pub use self::object::Object;

mod rt;
pub use self::rt::{Runtime, RuntimeGuard};

pub trait Signal<M>: Object {}

pub trait Handler<M>: Object {
    fn handle(&mut self, handle: Context<Self>, msg: M);
}
