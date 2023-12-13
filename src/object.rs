use crate::Handle;
use crate::handle::HandleGuard;
use core::any::Any;
use core::marker::PhantomData;

pub trait Object {
    #[allow(unused_variables)]
    fn start(&mut self, cx: Handle<Self>) {}

    cfg_rt!(
        fn spawn(self) -> crate::Handle<Self>
        where
            Self: Sized + 'static,
        {
            crate::Runtime::current().spawn(self)
        }
    );
}


pub trait AnyObject {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn start_any(&mut self, handle: HandleGuard);
}

impl<O: Object + 'static> AnyObject for O {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn start_any(&mut self, handle: HandleGuard) {
        let handle = Handle {
            guard: handle,
            _marker: PhantomData,
        };
        self.start(handle)
    }
}

