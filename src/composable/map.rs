use crate::{Composable, Context};
use std::{marker::PhantomData, sync::Arc};

/// View for the `View::map` method.
pub struct Map<V, F, M> {
    pub(super) view: V,
    pub(super) f: Arc<F>,
    pub(super) _marker: PhantomData<M>,
}

impl<V, F, M1, M2> Composable<M1> for Map<V, F, M2>
where
    V: Composable<M2>,
    F: Fn(M2) -> M1 + Send + Sync + 'static,
    M1: Send + 'static,
{
    type State = V::State;

    fn compose(&mut self, cx: &mut Context<M1>) -> Self::State {
        let f = self.f.clone();
        let send = cx.send.clone();
        let mut cx = Context::new(Arc::new(move |msg| send(f(msg))));

        self.view.compose(&mut cx)
    }

    fn recompose(&mut self, cx: &mut Context<M1>, state: &mut Self::State) {
        let f = self.f.clone();
        let send = cx.send.clone();
        let mut cx = Context::new(Arc::new(move |msg| send(f(msg))));

        self.view.recompose(&mut cx, state)
    }
}
