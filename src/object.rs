use crate::Context;

pub trait Object {
    #[allow(unused_variables)]
    fn start(&mut self, cx: Context<Self>) {}

    cfg_rt!(
        fn spawn(self) -> crate::Handle<Self>
        where
            Self: Sized + 'static,
        {
            crate::Runtime::current().spawn(self)
        }
    );
}
