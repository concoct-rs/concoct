use crate::{
    composer::Composer, modify::container::ContainerModifier, Modifier, Modify, Semantics, Widget,
};
use accesskit::{Node, NodeId};
use std::{any, mem, panic::Location};

#[track_caller]
pub fn container(
    mut modifier: Modifier<ContainerModifier, impl Modify<ContainerModifier> + 'static>,
    mut f: impl FnMut() + 'static,
) {
    let mut container_modifier = ContainerModifier::default();
    modifier.modify.modify(&mut container_modifier);

    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();

        let id = cx.id(location);
        let parent_children = mem::take(&mut cx.children);
        let parent_group_id = mem::replace(&mut cx.current_group_id, id.clone());
        drop(cx);

        f();

        let mut cx = composer.borrow_mut();
        cx.current_group_id = parent_group_id;
        let children = mem::replace(&mut cx.children, parent_children);

        if let Some(widget) = cx.get_mut::<ContainerWidget>(&id) {
            widget.modifier = container_modifier;
        } else {
            let widget = ContainerWidget {
                modifier: container_modifier,
                node_id: None,
                modify: Box::new(modifier.modify),
                f: Some(Box::new(f)),
            };
            cx.insert(id, widget, Some(children));
        }
    })
}

pub struct ContainerWidget {
    modifier: ContainerModifier,
    node_id: Option<NodeId>,
    pub modify: Box<dyn Modify<ContainerModifier>>,
    pub f: Option<Box<dyn FnMut()>>,
}

impl Widget for ContainerWidget {
    fn semantics(&mut self, semantics: &mut Semantics) {
        let id = if let Some(node_id) = self.node_id {
            semantics.end_group_update(node_id);
            node_id
        } else {
            let node = Node {
                role: self.modifier.role,
                ..Node::default()
            };

            let id = semantics.end_group_with_node(node, self.modifier.merge_descendants);
            self.node_id = Some(id);
            id
        };

        self.modify.semantics(id, semantics);
    }

    fn any(&self) -> &dyn any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
