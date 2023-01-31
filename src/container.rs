use crate::{composer::Composer, ContainerModifier, Modifier, Modify, Semantics, Widget};
use accesskit::{Node, NodeId, Role};
use std::{any, mem, panic::Location};

#[track_caller]
pub fn container(
    mut modifier: Modifier<ContainerModifier, impl Modify<ContainerModifier>>,
    role: Role,
    mut f: impl FnMut() + 'static,
) {
    let mut container_modifier = ContainerModifier {
        merge_descendants: false,
    };
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

        if let Some(_widget) = cx.get_mut::<ContainerWidget>(&id) {
        } else {
            let widget = ContainerWidget {
                role,
                node_id: None,
                merge: container_modifier.merge_descendants,
            };
            cx.insert(id, widget, Some(children));
        }
    })
}

struct ContainerWidget {
    role: Role,
    node_id: Option<NodeId>,
    merge: bool,
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

            let id = semantics.end_group_with_node(node, self.merge);
            self.node_id = Some(id);
        }
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
