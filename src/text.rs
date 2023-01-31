use std::any;

use crate::{composer::Composer, Semantics, Widget};
use accesskit::{Node, NodeId};

pub fn text(string: String) {
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        cx.insert_or_update(
            || TextWidget {
                text: string.clone(),
                node_id: None,
            },
            |widget| {
                widget.text = string.clone();
            },
        );
    })
}

pub struct TextWidget {
    text: String,
    node_id: Option<NodeId>,
}

impl Widget for TextWidget {
    fn semantics(&mut self, semantics: &mut Semantics) {
        let node = Node {
            value: Some(self.text.clone().into_boxed_str()),
            ..Node::default()
        };

        if let Some(node_id) = self.node_id {
            semantics.update(node_id, node);
        } else {
            let id = semantics.insert(node);
            self.node_id = Some(id);
        }
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
