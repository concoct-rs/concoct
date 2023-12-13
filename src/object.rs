use crate::handle::HandleGuard;
use crate::Handle;
use core::any::Any;
use core::marker::PhantomData;

/// A reactive object.
pub trait Object {
    /// Called after this object is started.
    /// 
    /// By default this does nothing.
    #[allow(unused_variables)]
    fn started(&mut self, cx: Handle<Self>) {}

    cfg_rt!(
        /// Start this object on the current runtime.
        fn start(self) -> crate::Handle<Self>
        where
            Self: Sized + 'static,
        {
            crate::Runtime::current().start(self)
        }
    );
}

/// A dynamic reactive object.
pub trait AnyObject {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn started_any(&mut self, handle: HandleGuard);
}

impl<O: Object + 'static> AnyObject for O {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn started_any(&mut self, handle: HandleGuard) {
        let handle = Handle {
            guard: handle,
            _marker: PhantomData,
        };
        self.started(handle)
    }
}
