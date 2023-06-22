use std::collections::HashMap;

#[derive(Default)]
pub struct Context {
    next_id: usize,
    unused_ids: Vec<usize>,
    handlers: HashMap<usize, Box<dyn FnMut()>>,
}

impl Context {
    pub fn handle(&mut self, id: usize) {
        self.handlers.get_mut(&id).unwrap()();
    }
}

pub trait View {
    fn view(&mut self, cx: &mut Context);
}

impl View for String {
    fn view(&mut self, cx: &mut Context) {
        dbg!(self);
    }
}

pub struct Id {
    cell: Option<usize>,
}

impl Id {
    pub fn get(&mut self, cx: &mut Context) -> usize {
        if let Some(id) = self.cell {
            id
        } else {
            let id = if let Some(id) = cx.unused_ids.pop() {
                id
            } else {
                cx.next_id
            };

            self.cell = Some(id);
            id
        }
    }
}

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

#[cfg(test)]
mod tests {
    use crate::{Button, Context, View};

    #[test]
    fn f() {
        let mut cx = Context::default();
        let mut button = Button::new(String::from("Hello World!"), || {
            dbg!("Press!");
        });

        button.view(&mut cx);
        cx.handle(0);
    }
}
