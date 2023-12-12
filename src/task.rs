use crate::{Handle, Runtime};

pub trait Task {
    fn spawn(self) -> Handle<Self>
    where
        Self: Sized + 'static,
    {
        Runtime::current().spawn(self)
    }
}
