use crate::{
    composer::{Composer, Id},
    Semantics, Widget,
};
use accesskit::{Node, NodeId, Role};
use std::{any, mem, panic::Location};

#[track_caller]
pub fn container(role: Role, mut f: impl FnMut() + 'static) {
    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();

        let id = cx.id(location);
        let parent_children = mem::take(&mut cx.children);
        drop(cx);

        f();

        let mut cx = composer.borrow_mut();
        let children = mem::replace(&mut cx.children, parent_children);

        if let Some(widget) = cx.get_mut::<ContainerWidget>(&id) {
        } else {
            let widget = ContainerWidget {
                role,
                node_id: None,
            };
            cx.insert(id, widget, Some(children));
        }
    })
}

struct ContainerStartWidget {}

impl Widget for ContainerStartWidget {
    fn semantics(&mut self, semantics: &mut Semantics) {
        semantics.start_group();
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}

struct ContainerWidget {
    role: Role,
    node_id: Option<NodeId>,
}

impl Widget for ContainerWidget {
    fn semantics(&mut self, semantics: &mut Semantics) {
        if let Some(node_id) = self.node_id {
            semantics.end_group_update(node_id);
        } else {
            let node = Node {
                role: self.role,
                ..Node::default()
            };

            let id = semantics.end_group_with_node(node);
            self.node_id = Some(id);
        }
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
