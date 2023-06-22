use std::collections::HashMap;

pub mod view;
pub use view::View;

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

#[cfg(test)]
mod tests {
    use crate::{view::Button, Context, View};

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
