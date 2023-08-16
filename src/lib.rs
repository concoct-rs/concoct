mod renderer;

pub use renderer::{Event, Renderer};
use view::{Id, View};

pub mod view;

pub struct EventHandler<F, V> {
    on_event: F,
    view: V,
}

impl<F, V> EventHandler<F, V> {
    pub fn new(on_event: F, view: V) -> Self {
        Self { on_event, view }
    }
}

impl<T, A, F, V> View<T, A> for EventHandler<F, V>
where
    V: View<T, A>,
    F: FnMut(&mut T),
{
    type State = V::State;

    fn build(&mut self, cx: &mut view::BuildContext) -> (Id, Self::State) {
        self.view.build(cx)
    }

    fn rebuild(&mut self, cx: &mut view::BuildContext, old: &mut Self) {
        self.view.rebuild(cx, &mut old.view)
    }

    fn layout(&mut self, cx: &mut view::LayoutContext) {
        self.view.layout(cx)
    }

    fn paint(&mut self, taffy: &taffy::Taffy, canvas: &mut skia_safe::Canvas) {
        self.view.paint(taffy, canvas)
    }

    fn message(&mut self, state: &mut T, id_path: &[Id], message: &dyn std::any::Any) {
        (self.on_event)(state);
        self.view.message(state, id_path, message)
    }
}
