use super::{Platform, View};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub fn once<V, P>(view: V) -> Lazy<(), V>
where
    V: View<P>,
    P: Platform,
{
    Lazy {
        view,
        hash: 0,
        _marker: PhantomData,
    }
}

/// Lazy-loaded view.
/// The child view will only be rebuild if the input has changed.
pub fn lazy<T, V, P>(input: &T, view: V) -> Lazy<T, V>
where
    T: Hash,
    V: View<P>,
    P: Platform,
{
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);

    Lazy {
        view,
        hash: hasher.finish(),
        _marker: PhantomData,
    }
}

/// View for the [`lazy`] function.
pub struct Lazy<T, V> {
    view: V,
    hash: u64,
    _marker: PhantomData<T>,
}

impl<P, T, V> View<P> for Lazy<T, V>
where
    T: Hash,
    V: View<P>,
    P: Platform,
{
    type State = (u64, V::State);

    fn build(self, cx: &mut P) -> Self::State {
        let child_state = self.view.build(cx);
        (self.hash, child_state)
    }

    fn rebuild(self, cx: &mut P, state: &mut Self::State) {
        if self.hash != state.0 {
            state.0 = self.hash;
            self.view.rebuild(cx, &mut state.1)
        }
    }

    fn remove(cx: &mut P, state: &mut Self::State) {
        V::remove(cx, &mut state.1)
    }
}
