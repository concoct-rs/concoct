use crate::{composer::Composer, ContainerModifier, Modifier, Modify, Semantics, Widget};
use accesskit::{Node, NodeId};
use std::{any, mem, panic::Location};

#[track_caller]
pub fn container(
    mut modifier: Modifier<ContainerModifier, impl Modify<ContainerModifier>>,
    mut f: impl FnMut() + 'static,
) {
    let mut container_modifier = ContainerModifier::default();
    modifier.modify.modify(&mut container_modifier);

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
            widget.modifier = container_modifier;
        } else {
            let widget = ContainerWidget {
                modifier: container_modifier,
                node_id: None,
            };
            cx.insert(id, widget, Some(children));
        }
    })
}

struct ContainerWidget {
    modifier: ContainerModifier,
    node_id: Option<NodeId>,
}

impl Widget for ContainerWidget {
    fn semantics(&mut self, semantics: &mut Semantics) {
        if let Some(node_id) = self.node_id {
            semantics.end_group_update(node_id);
        } else {
            let node = Node {
                role: self.modifier.role,
                ..Node::default()
            };

            let id = semantics.end_group_with_node(node, self.modifier.merge_descendants);
            self.node_id = Some(id);
        }
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
