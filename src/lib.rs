use std::collections::HashMap;

pub mod view;
use slotmap::DefaultKey;
use taffy::{
    prelude::{Node, Size},
    style::{AvailableSpace, FlexDirection, Style},
    style_helpers::{points, TaffyMaxContent},
    Taffy,
};
pub use view::View;

#[derive(Default)]
pub struct Context {
    next_id: usize,
    unused_ids: Vec<usize>,
    handlers: HashMap<usize, Box<dyn FnMut()>>,
    taffy: Taffy,
    root: Option<DefaultKey>,
    children: Vec<Node>,
}

impl Context {
    pub fn handle(&mut self, id: usize) {
        self.handlers.get_mut(&id).unwrap()();
    }

    pub fn layout(&mut self) {
        let root = if let Some(root) = self.root {
            self.taffy.set_children(root, &self.children).unwrap();
            root
        } else {
            let style = Style {
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: points(800.0),
                    height: points(600.0),
                },
                ..Default::default()
            };
            let root = self.taffy.new_with_children(style, &self.children).unwrap();
            self.root = Some(root);
            root
        };

        self.taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
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
    use crate::{
        view::{Button, Text},
        Context, View,
    };

    #[test]
    fn f() {
        let mut cx = Context::default();
        let mut text = Text::new("Test");
        text.view(&mut cx);

        cx.layout();
        dbg!(cx.taffy.layout(cx.children.first().unwrap().clone()));
    }
}
