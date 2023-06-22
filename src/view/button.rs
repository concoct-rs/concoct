use crate::{Context, Id, View};

pub struct Button<V, F> {
    id: Id,
    view: V,
    on_press: Option<F>,
}

impl<V, F> Button<V, F> {
    pub fn new(view: V, on_press: F) -> Self {
        Self {
            id: Id { cell: None },
            view,
            on_press: Some(on_press),
        }
    }
}

impl<V, F> View for Button<V, F>
where
    V: View,
    F: FnMut() + 'static,
{
    fn view(&mut self, cx: &mut Context) {
        if let Some(on_press) = self.on_press.take() {
            let id = self.id.get(cx);
            cx.handlers.insert(id, Box::new(on_press));
        }

        self.view.view(cx);
    }
}
