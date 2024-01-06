use crate::Composable;
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn lazy<M, C>(value: &impl Hash, composable: C) -> Lazy<C>
where
    C: Composable<M>,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let hash = hasher.finish();

    Lazy { hash, composable }
}

pub struct Lazy<C> {
    hash: u64,
    composable: C,
}

impl<M, C> Composable<M> for Lazy<C>
where
    C: Composable<M>,
{
    type State = (u64, C::State);

    fn compose(&mut self, cx: &mut crate::Context<M>) -> Self::State {
        let state = self.composable.compose(cx);
        (self.hash, state)
    }

    fn recompose(&mut self, cx: &mut crate::Context<M>, state: &mut Self::State) {
        if self.hash != state.0 {
            state.0 = self.hash;
            self.composable.recompose(cx, &mut state.1);
        }
    }
}
