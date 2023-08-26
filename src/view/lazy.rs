use super::View;

pub fn lazy<T, V>(input: T, view: V) -> Lazy<T, V> {
    Lazy { input, view }
}

pub struct Lazy<T, V> {
    input: T,
    view: V,
}

impl<M, T, V> View<M> for Lazy<T, V>
where
    T: PartialEq,
    V: View<M>,
{
    type State = (T, V::State);

    fn build(self, cx: &mut crate::Context<M>) -> Self::State {
        let child_state = self.view.build(cx);
        (self.input, child_state)
    }

    fn rebuild(self, cx: &mut crate::Context<M>, state: &mut Self::State) {
        if self.input != state.0 {
            self.view.rebuild(cx, &mut state.1)
        }
    }

    fn remove(cx: &mut crate::Context<M>, state: &mut Self::State) {
        V::remove(cx, &mut state.1)
    }
}
