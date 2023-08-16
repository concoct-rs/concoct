use super::View;

pub fn remember<F, S, G, V>(make_state: F, make_view: G) -> Remember<F, S, G, V> {
    Remember::new(make_state, make_view)
}

pub struct Remember<F, S, G, V> {
    make_state: F,
    state: Option<S>,
    make_view: G,
    view: Option<V>,
}

impl<F, S, G, V> Remember<F, S, G, V> {
    pub fn new(make_state: F, make_view: G) -> Self {
        Self {
            make_state,
            state: None,
            make_view,
            view: None,
        }
    }
}

impl<T, A, F, S, G, V> View<T, A> for Remember<F, S, G, V>
where
    F: FnMut() -> S,
    G: FnMut(&mut S) -> V,
    V: View<S, A>,
{
    type State = V::State;

    fn build(&mut self, cx: &mut super::BuildContext) -> (super::Id, Self::State) {
        let mut state = (self.make_state)();
        let mut view = (self.make_view)(&mut state);
        self.state = Some(state);
        let (id, s) = view.build(cx);
        self.view = Some(view);
        (id, s)
    }

    fn rebuild(&mut self, cx: &mut super::BuildContext, old: &mut Self) {
        let mut state = old.state.take().unwrap();
        let mut view = (self.make_view)(&mut state);
        self.state = Some(state);
        view.rebuild(cx, old.view.as_mut().unwrap());
        self.view = Some(view);
    }

    fn layout(&mut self, cx: &mut super::LayoutContext, id: super::Id) {
        self.view.as_mut().unwrap().layout(cx, id)
    }

    fn paint(&mut self, taffy: &taffy::Taffy, canvas: &mut skia_safe::Canvas) {
        self.view.as_mut().unwrap().paint(taffy, canvas)
    }

    fn message(&mut self, _state: &mut T, id_path: &[super::Id], message: &dyn std::any::Any) {
        self.view
            .as_mut()
            .unwrap()
            .message(self.state.as_mut().unwrap(), id_path, message)
    }
}
