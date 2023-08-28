use super::View;

pub fn lazy<T, V>(input: T, view: V) -> Lazy<T, V> {
    Lazy { input, view }
}

pub struct Lazy<T, V> {
    input: T,
    view: V,
}

impl<E, T, V> View<Web<E>> for Lazy<T, V>
where
    T: PartialEq,
    V: View<Web<E>>,
{
    type State = (T, V::State);

    fn build(self, cx: &mut crate::Context<E>) -> Self::State {
        let child_state = self.view.build(cx);
        (self.input, child_state)
    }

    fn rebuild(self, cx: &mut crate::Context<E>, state: &mut Self::State) {
        if self.input != state.0 {
            state.0 = self.input;
            self.view.rebuild(cx, &mut state.1)
        }
    }

    fn remove(cx: &mut crate::Context<E>, state: &mut Self::State) {
        V::remove(cx, &mut state.1)
    }
}
