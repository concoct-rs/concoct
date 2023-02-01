use crate::{
    composer::{Composer, WidgetNode},
    modify::container::ContainerModifier,
    Modifier, Modify, Semantics, Widget,
};
use accesskit::{Node, NodeId};
use std::{any, panic::Location};

#[track_caller]
pub fn container(
    mut modifier: Modifier<ContainerModifier, impl Modify<ContainerModifier> + 'static>,
    mut f: impl FnMut() + 'static,
) {
    let mut container_modifier = ContainerModifier::default();
    modifier.modify.modify(&mut container_modifier);

    let location = Location::caller();
    Composer::with(|composer| {
        let cx = composer.borrow();
        let id = cx.id(location);
        drop(cx);

        let (children, removed) = Composer::group(&id, &mut f);

        let mut cx = composer.borrow_mut();
        if let Some(node) = cx.widgets.get_mut(&id) {
            let widget: &mut ContainerWidget = node.widget.any_mut().downcast_mut().unwrap();
            widget.modifier = container_modifier;
            widget.removed = removed;
            node.children = Some(children);

            cx.children.push(id);
        } else {
            let widget = ContainerWidget {
                modifier: container_modifier,
                node_id: None,
                modify: Box::new(modifier.modify),
                f: Some(Box::new(f)),
                removed: None,
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
    removed: Option<Vec<WidgetNode>>,
}

impl Widget for ContainerWidget {
    fn semantics(&mut self, semantics: &mut Semantics) {
        if let Some(removed) = &mut self.removed {
            for child in removed {
                child.widget.remove(semantics);
            }
        }

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

    fn paint(&mut self, _semantics: &Semantics, _canvas: &mut skia_safe::Canvas) {}

    fn remove(&mut self, semantics: &mut Semantics) {
        if let Some(node_id) = self.node_id {
            semantics.remove(node_id);
        }
    }

    fn any(&self) -> &dyn any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
