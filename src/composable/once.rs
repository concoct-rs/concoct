use crate::{Composable, Context};

pub fn once<C, M>(composable: C) -> Once<C>
where
    C: Composable<M>,
{
    Once { composable }
}

pub struct Once<C> {
    composable: C,
}

impl<M, C> Composable<M> for Once<C>
where
    C: Composable<M>,
{
    type State = C::State;

    fn compose(&mut self, cx: &mut Context<M>) -> Self::State {
        self.composable.compose(cx)
    }

    fn recompose(&mut self, _cx: &mut Context<M>, _state: &mut Self::State) {}
}
