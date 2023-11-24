use crate::BuildContext;

/// Composable object that handles diffing.
pub trait Composable {
    type State: 'static;

    fn build(&mut self, cx: &mut BuildContext) -> Self::State;

    fn rebuild(&mut self, state: &mut Self::State);
}

impl<F, C> Composable for F
where
    F: FnMut() -> C + Clone + 'static,
    C: Composable + 'static,
{
    type State = ();

    fn build(&mut self, cx: &mut BuildContext) -> Self::State {
        let mut f = self.clone();
        cx.insert(Box::new(move || Box::new(f())));
    }

    fn rebuild(&mut self, _state: &mut Self::State) {}
}

impl<A: Composable, B: Composable> Composable for (A, B) {
    type State = (A::State, B::State);

    fn build(&mut self, cx: &mut BuildContext) -> Self::State {
        ((self.0).build(cx), (self.1).build(cx))
    }

    fn rebuild(&mut self, state: &mut Self::State) {
        (self.0).rebuild(&mut state.0);
        (self.1).rebuild(&mut state.1);
    }
}

impl Composable for () {
    type State = ();

    fn build(&mut self, _cx: &mut BuildContext) -> Self::State {}

    fn rebuild(&mut self, _state: &mut Self::State) {}
}