use crate::{Handle, Runtime, handle::HandleRef};

pub trait Object {
    #[allow(unused_variables)]
    fn start(&mut self, handle: HandleRef<Self>) {}

    fn spawn(self) -> Handle<Self>
    where
        Self: Sized + 'static,
    {
        Runtime::current().spawn(self)
    }
}
