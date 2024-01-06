use crate::Context;
use std::{marker::PhantomData, sync::Arc};

mod from_fn;
pub use self::from_fn::{from_fn, FromFn};

mod lazy;
pub use self::lazy::{lazy, Lazy};

mod map;
pub use self::map::Map;

mod once;
pub use self::once::{once, Once};

pub trait Composable<M> {
    type State;

    fn compose(&mut self, cx: &mut Context<M>) -> Self::State;

    fn recompose(&mut self, cx: &mut Context<M>, state: &mut Self::State);

    fn map<F, M1>(self, f: F) -> Map<Self, F, M>
    where
        Self: Sized,
        F: Fn(M) -> M1 + 'static,
        M1: 'static,
    {
        Map {
            view: self,
            f: Arc::new(f),
            _marker: PhantomData,
        }
    }
}

impl<M> Composable<M> for () {
    type State = ();

    fn compose(&mut self, _cx: &mut Context<M>) -> Self::State {}

    fn recompose(&mut self, _cx: &mut Context<M>, _state: &mut Self::State) {}
}

impl<M, C1: Composable<M>, C2: Composable<M>> Composable<M> for (C1, C2) {
    type State = (C1::State, C2::State);

    fn compose(&mut self, cx: &mut Context<M>) -> Self::State {
        (self.0.compose(cx), self.1.compose(cx))
    }

    fn recompose(&mut self, cx: &mut Context<M>, state: &mut Self::State) {
        self.0.recompose(cx, &mut state.0);
        self.1.recompose(cx, &mut state.1);
    }
}
