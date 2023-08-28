use super::{Platform, View};

/// Lazy-loaded view.
/// The child view will only be rebuild if the input has changed.
pub fn lazy<T, V, P>(input: T, view: V) -> Lazy<T, V>
where
    T: PartialEq,
    V: View<P>,
    P: Platform,
{
    Lazy { input, view }
}

/// View for the [`lazy`] function.
pub struct Lazy<T, V> {
    input: T,
    view: V,
}

impl<P, T, V> View<P> for Lazy<T, V>
where
    T: PartialEq,
    V: View<P>,
    P: Platform,
{
    type State = (T, V::State);

    fn build(self, cx: &mut P) -> Self::State {
        let child_state = self.view.build(cx);
        (self.input, child_state)
    }

    fn rebuild(self, cx: &mut P, state: &mut Self::State) {
        if self.input != state.0 {
            state.0 = self.input;
            self.view.rebuild(cx, &mut state.1)
        }
    }

    fn remove(cx: &mut P, state: &mut Self::State) {
        V::remove(cx, &mut state.1)
    }
}
