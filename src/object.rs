use crate::{Context, Handle};
use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;

/// A reactive object.
pub trait Object {
    /// Called when the object is first started.
    #[allow(unused_variables)]
    fn started(cx: &mut Context<Self>)
    where
        Self: Sized + 'static,
    {
    }

    /// Start this object and create a handle.
    fn start(self) -> Handle<Self>
    where
        Self: Sized + 'static,
    {
        let handle = Handle::new(self);
        Self::started(&mut handle.cx());
        handle
    }

    /// Create a handle to a new default object.
    fn create() -> Handle<Self>
    where
        Self: Default + 'static,
    {
        Self::default().start()
    }
}

impl<O: Object + ?Sized> Object for &mut O {}

impl<F> Object for Box<F> where F: Object + ?Sized {}

impl<P> Object for Pin<P>
where
    P: DerefMut,
    <P as Deref>::Target: Object,
{
}
