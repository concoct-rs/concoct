use crate::{
    composer::{Composer, Id, WidgetNode},
    modify::ModifyExt,
    semantics::LayoutNode,
    Modifier, Modify, Semantics, Widget,
};
use accesskit::{kurbo::Rect, Node, NodeId, Role};
use skia_safe::Canvas;
use std::{any, panic::Location};
use taffy::style::FlexDirection;

pub mod modifier;
use self::modifier::ContainerConfig;
pub use modifier::ContainerModifier;

#[track_caller]
pub fn row(modifier: impl Modify<ContainerConfig> + 'static, composable: impl FnMut() + 'static) {
    container(
        Modifier
            .role(Role::Row)
            .flex_direction(FlexDirection::Row)
            .chain(modifier),
        composable,
    )
}

#[track_caller]
pub fn column(
    modifier: impl Modify<ContainerConfig> + 'static,
    composable: impl FnMut() + 'static,
) {
    container(
        Modifier
            .role(Role::Column)
            .flex_direction(FlexDirection::Column)
            .chain(modifier),
        composable,
    )
}

#[track_caller]
pub fn container(
    mut modifier: impl Modify<ContainerConfig> + 'static,
    mut composable: impl FnMut() + 'static,
) {
    let mut container_modifier = ContainerConfig::default();
    modifier.modify(&mut container_modifier);

    let location = Location::caller();
    Composer::with(|composer| {
        let cx = composer.borrow();
        let id = cx.id(location);
        drop(cx);

        let (children, removed) = Composer::group(&id, &mut composable);

        let mut cx = composer.borrow_mut();
        if let Some(node) = cx.widgets.get_mut(&id) {
            let widget: &mut ContainerWidget = node.as_mut();
            widget.modifier = container_modifier;
            widget.removed = removed;
            widget.children = children.clone();
            node.children = Some(children);

            cx.children.push(id);
        } else {
            let widget = ContainerWidget {
                modifier: container_modifier,
                node_id: None,
                modify: Box::new(modifier),
                f: Some(Box::new(composable)),
                removed: None,
                layout_id: None,
                children: children.clone(),
            };
            cx.insert(id, widget, Some(children));
        }
    })
}

pub struct ContainerWidget {
    modifier: ContainerConfig,
    node_id: Option<NodeId>,
    pub modify: Box<dyn Modify<ContainerConfig>>,
    pub f: Option<Box<dyn FnMut()>>,
    removed: Option<Vec<WidgetNode>>,
    pub layout_id: Option<LayoutNode>,
    pub children: Vec<Id>,
}

impl Widget for ContainerWidget {
    fn layout(&mut self, semantics: &mut Semantics) {
        let layout_children = semantics.layout_children.pop().unwrap();

        if let Some(layout_id) = self.layout_id {
            semantics
                .taffy
                .set_children(layout_id, &layout_children)
                .unwrap();
            semantics
                .layout_children
                .last_mut()
                .unwrap()
                .push(layout_id);
        } else {
            let layout_id =
                semantics.insert_layout_with_children(self.modifier.style, &layout_children);
            self.layout_id = Some(layout_id);
        }
    }

    fn semantics(&mut self, semantics: &mut Semantics) {
        if let Some(removed) = &mut self.removed {
            for child in removed {
                child.widget.remove(semantics);
            }
        }

        let layout = semantics.layout(self.layout_id.unwrap());
        let bounds = Rect::new(
            layout.location.x as _,
            layout.location.y as _,
            (layout.location.x + layout.size.width) as _,
            (layout.location.y + layout.size.height) as _,
        );
        let node = Node {
            role: self.modifier.role,
            bounds: Some(bounds),
            ..Node::default()
        };

        let id = if let Some(node_id) = self.node_id {
            semantics.end_group_update(node_id, node);
            node_id
        } else {
            let id = semantics.end_group_with_node(node, self.modifier.merge_descendants);
            self.node_id = Some(id);
            id
        };

        self.modify.semantics(id, semantics);
    }

    fn paint(&mut self, _semantics: &Semantics, _canvas: &mut Canvas) {}

    fn remove(&mut self, semantics: &mut Semantics) {
        if let Some(node_id) = self.node_id {
            semantics.remove(node_id);

            Composer::with(|composer| {
                for child_id in &mut self.children {
                    let mut node = {
                        let mut cx = composer.borrow_mut();
                        cx.widgets.remove(&child_id).unwrap()
                    };

                    node.widget.remove(semantics);
                }
            });

            self.modify.remove(node_id, semantics)
        }
    }

    fn any(&self) -> &dyn any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
