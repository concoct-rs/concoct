use crate::{
    composer::{Composer, Id},
    dimension::{Padding, Size},
    semantics::LayoutNode,
    Modifier, Modify, Semantics, View, Widget,
};
use accesskit::{kurbo::Rect, Node, NodeId, Role};
use skia_safe::Canvas;
use std::{any, panic::Location};
use taffy::style::{AlignItems, Dimension, FlexDirection, JustifyContent, Style};

struct ContainerConfig {
    merge_descendants: bool,
    role: Role,
    style: Style,
}

pub struct Container<C, M> {
    content: C,
    modifier: M,
    config: ContainerConfig,
}

impl<C> Container<C, Modifier> {
    #[track_caller]
    pub fn build(content: C, role: Role) -> Self {
        Self {
            content,
            modifier: Modifier,
            config: ContainerConfig {
                merge_descendants: false,
                role,
                style: Style::default(),
            },
        }
    }

    #[track_caller]
    pub fn build_row(content: C) -> Self
    where
        C: FnMut(),
    {
        Self::build(content, Role::Row).flex_direction(FlexDirection::Row)
    }

    #[track_caller]
    pub fn build_column(content: C) -> Self
    where
        C: FnMut(),
    {
        Self::build(content, Role::Column).flex_direction(FlexDirection::Column)
    }

    #[track_caller]
    pub fn row(content: C)
    where
        C: FnMut() + 'static,
    {
        Self::build_row(content).view()
    }

    #[track_caller]
    pub fn column(content: C)
    where
        C: FnMut() + 'static,
    {
        Self::build_column(content).view()
    }
}

impl<C, M> Container<C, M> {
    pub fn modifier<M2>(self, modifier: M2) -> Container<C, M2> {
        Container {
            content: self.content,
            modifier,
            config: self.config,
        }
    }

    pub fn merge_descendants(mut self) -> Self {
        self.config.merge_descendants = true;
        self
    }

    pub fn align_items(mut self, align_items: AlignItems) -> Self {
        self.config.style.align_items = align_items;
        self
    }

    pub fn flex_basis(mut self, dimension: Dimension) -> Self {
        self.config.style.flex_basis = dimension;
        self
    }

    pub fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        self.config.style.flex_direction = flex_direction;
        self
    }

    pub fn flex_grow(mut self, value: f32) -> Self {
        self.config.style.flex_grow = value;
        self
    }

    pub fn flex_shrink(mut self, value: f32) -> Self {
        self.config.style.flex_shrink = value;
        self
    }

    pub fn gap(mut self, size: Size) -> Self {
        self.config.style.gap = size.into();
        self
    }

    pub fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.config.style.justify_content = justify_content;
        self
    }

    pub fn margin(mut self, rect: taffy::prelude::Rect<Dimension>) -> Self {
        self.config.style.margin = rect;
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.config.style.padding = padding.rect;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.config.style.size = size.into();
        self
    }

    pub fn role(mut self, role: Role) -> Self {
        self.config.role = role;
        self
    }
}

impl<C, M> View for Container<C, M>
where
    C: FnMut() + 'static,
    M: Modify + 'static,
{
    #[track_caller]
    fn view(mut self) {
        let location = Location::caller();
        Composer::with(|composer| {
            let id = composer.borrow_mut().id(location);

            let children = Composer::group(&id, &mut self.content);

            let mut cx = composer.borrow_mut();
            if let Some(node) = cx.widgets.get_mut(&id) {
                let widget: &mut ContainerWidget = node.as_mut();
                widget.config = self.config;
                widget.content = Some(Box::new(self.content));
                widget.modifier = Box::new(self.modifier);

                widget.children = children.clone();

                node.children = Some(children);
                cx.children.push(id);
            } else {
                let widget = ContainerWidget {
                    config: self.config,
                    content: Some(Box::new(self.content)),
                    modifier: Box::new(self.modifier),
                    node_id: None,

                    layout_id: None,
                    children: children.clone(),
                };
                cx.insert(id, widget, Some(children));
            }
        })
    }
}

pub struct ContainerWidget {
    config: ContainerConfig,
    pub content: Option<Box<dyn FnMut()>>,
    pub modifier: Box<dyn Modify>,
    node_id: Option<NodeId>,

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
                semantics.insert_layout_with_children(self.config.style, &layout_children);
            self.layout_id = Some(layout_id);
        }
    }

    fn semantics(&mut self, semantics: &mut Semantics) {
        let layout = semantics.layout(self.layout_id.unwrap());
        let bounds = Rect::new(
            layout.location.x as _,
            layout.location.y as _,
            (layout.location.x + layout.size.width) as _,
            (layout.location.y + layout.size.height) as _,
        );
        let node = Node {
            role: self.config.role,
            bounds: Some(bounds),
            ..Node::default()
        };

        let id = if let Some(node_id) = self.node_id {
            semantics.end_group_update(node_id, node);
            node_id
        } else {
            let id = semantics.end_group_with_node(node, self.config.merge_descendants);
            self.node_id = Some(id);
            id
        };

        self.modifier.semantics(id, semantics);
    }

    fn paint(&mut self, _semantics: &Semantics, _canvas: &mut Canvas) {}

    fn remove(&mut self, semantics: &mut Semantics) {
        if let Some(node_id) = self.node_id {
            semantics.remove(node_id);

            Composer::with(|composer| {
                for child_id in &mut self.children {
                    let mut cx = composer.borrow_mut();
                    if let Some(mut node) = cx.widgets.remove(&child_id) {
                        drop(cx);
                        node.widget.remove(semantics);
                    }
                }
            });

            self.modifier.remove(node_id, semantics)
        }
    }

    fn any(&self) -> &dyn any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
