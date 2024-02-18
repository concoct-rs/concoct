use super::Task;
use std::marker::PhantomData;

pub fn from_fn<F, M, O>(f: F) -> FromFn<F, M>
where
    F: FnMut(&mut M) -> O,
{
    FromFn {
        f,
        _marker: PhantomData,
    }
}

pub struct FromFn<F, M> {
    f: F,
    _marker: PhantomData<M>,
}

impl<F, M, O> Task<M> for FromFn<F, M>
where
    F: FnMut(&mut M) -> O,
{
    type Output = O;

    type State = ();

    fn build(&mut self, _cx: &super::Scope<M, ()>, model: &mut M) -> (Self::Output, Self::State) {
        ((self.f)(model), ())
    }

    fn rebuild(
        &mut self,
        _cx: &super::Scope<M, ()>,
        model: &mut M,
        _state: &mut Self::State,
    ) -> Self::Output {
        (self.f)(model)
    }
}
