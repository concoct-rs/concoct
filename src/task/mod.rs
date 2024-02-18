use std::rc::Rc;

mod from_fn;
pub use self::from_fn::{from_fn, FromFn};

mod then;
pub use self::then::Then;

pub struct Context<M, A> {
    waker: Rc<dyn Fn(Rc<dyn Fn(&mut M) -> Option<A>>)>,
}

impl<M, A> Context<M, A> {
    pub fn update(&self, f: impl Fn(&mut M) -> Option<A> + 'static) {
        (self.waker)(Rc::new(f))
    }
}

pub trait Task<M, A = ()> {
    type Output;

    type State;

    fn build(&mut self, cx: &Context<M, A>, model: &mut M) -> (Self::Output, Self::State);

    fn rebuild(
        &mut self,
        cx: &Context<M, A>,
        model: &mut M,
        state: &mut Self::State,
    ) -> Self::Output;

    fn then<F, T>(self, f: F) -> Then<Self, F, T, M>
    where
        Self: Sized + 'static,
        F: FnMut(&mut M, Self::Output) -> T + 'static,
        T: Task<M> + 'static,
        M: 'static,
    {
        Then::new(self, f)
    }
}

impl<M> Task<M> for () {
    type Output = ();

    type State = ();

    fn build(&mut self, _cx: &Context<M, ()>, _model: &mut M) -> (Self::Output, Self::State) {
        ((), ())
    }

    fn rebuild(
        &mut self,
        _cx: &Context<M, ()>,
        _model: &mut M,
        _state: &mut Self::State,
    ) -> Self::Output {
    }
}
