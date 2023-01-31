use crate::{composer::Composer, Semantics, Widget};
use accesskit::{Node, NodeId};
use std::{any, panic::Location};

#[track_caller]
pub fn text(string: String) {
    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        if let Some(widget) = cx.get_mut::<TextWidget>(&id) {
            widget.text = string.clone();
        } else {
            let widget = TextWidget {
                text: string.clone(),
                node_id: None,
            };
            cx.insert(id, widget, None);
        }
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
