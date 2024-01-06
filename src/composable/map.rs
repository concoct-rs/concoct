use std::{any::Any, marker::PhantomData, sync::Arc};

use super::Composable;
use crate::Context;

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
        let mapper: Arc<dyn Fn(M2) -> Box<dyn Any + Send> + Send + Sync> =
            Arc::new(move |msg| Box::new((f)(msg)));
        let mut cx = Context {
            mapper: Some(mapper),
            tx: cx.tx.clone(),
        };
        self.view.compose(&mut cx)
    }

    fn recompose(&mut self, cx: &mut Context<M1>, state: &mut Self::State) {
        let f = self.f.clone();
        let mapper: Arc<dyn Fn(M2) -> Box<dyn Any + Send> + Send + Sync> =
            Arc::new(move |msg| Box::new((f)(msg)));
        let mut cx = Context {
            mapper: Some(mapper),
            tx: cx.tx.clone(),
        };
        self.view.recompose(&mut cx, state)
    }
}
