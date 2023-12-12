use crate::{Context, Handle, Runtime};

pub trait Object {
    #[allow(unused_variables)]
    fn start(&mut self, cx: Context<Self>) {}

    fn spawn(self) -> Handle<Self>
    where
        Self: Sized + 'static,
    {
        Runtime::current().spawn(self)
    }
}
