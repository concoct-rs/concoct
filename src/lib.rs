mod handle;
pub use self::handle::{Handle, HandleRef};

mod object;
pub use self::object::Object;

mod rt;
pub use rt::Runtime;

pub trait Signal<M>: Object {}

pub trait Handler<M>: Object {
    fn handle(&mut self, handle: HandleRef<Self>, msg: M);
}
