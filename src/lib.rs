mod handle;
pub use self::handle::Handle;

mod task;
pub use self::task::Task;

mod rt;
pub use rt::Runtime;

pub trait Handler<M>: Task {
    fn handle(&mut self, msg: M);
}
