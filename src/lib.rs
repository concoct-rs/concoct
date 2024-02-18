use std::rc::Rc;

pub struct Context<M, A> {
    waker: Rc<dyn Fn(Rc<dyn Fn(&mut M) -> Option<A>>)>,
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
}
