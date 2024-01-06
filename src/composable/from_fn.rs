use super::Composable;
use crate::Context;

pub fn from_fn<F, M>(f: F) -> FromFn<F>
where
    F: FnMut(&mut Context<M>),
{
    FromFn { f }
}

pub struct FromFn<F> {
    f: F,
}

impl<M, F> Composable<M> for FromFn<F>
where
    F: FnMut(&mut Context<M>),
{
    type State = ();

    fn compose(&mut self, cx: &mut Context<M>) -> Self::State {
        (self.f)(cx)
    }

    fn recompose(&mut self, cx: &mut Context<M>, _state: &mut Self::State) {
        (self.f)(cx)
    }
}
