use crate::{semantics::LayoutNode, Modifier, Modify, Widget};
use accesskit::{Node, NodeId, Role};
use taffy::style::Style;

use super::widget;

pub struct Element<M> {
    modifier: M,
    style: Style,
    layout_node: Option<LayoutNode>,
    node_id: Option<NodeId>,
}

impl Element<Modifier> {
    pub fn build() -> Self {
        Self {
            modifier: Modifier,
            style: Style::default(),
            layout_node: None,
            node_id: None,
        }
    }

    #[track_caller]
    pub fn new() {
        Self::build().view()
    }
}

impl<M: Modify + 'static> Element<M> {
    #[track_caller]
    pub fn view(self) {
        widget(
            self,
            |me| me,
            |me, node| {
                let widget: &mut Self = node.as_mut();
                widget.modifier = me.modifier;
                widget.style = me.style;
            },
        );
    }
}

impl<M: Modify + 'static> Widget for Element<M> {
    fn layout(&mut self, semantics: &mut crate::Semantics) {
        if let Some(layout_node) = self.layout_node {
            semantics.taffy.set_style(layout_node, self.style).unwrap()
        } else {
            self.layout_node = Some(semantics.insert_layout_with_children(self.style, &[]))
        }
    }

    fn semantics(&mut self, semantics: &mut crate::Semantics) {
        let node = Node {
            role: Role::Canvas,
            ..Node::default()
        };

        let node_id = if let Some(node_id) = self.node_id {
            semantics.update(node_id, node);
            node_id
        } else {
            let node_id = semantics.insert(node);
            self.node_id = Some(node_id);
            node_id
        };

        self.modifier.semantics(node_id, semantics);
    }

    fn paint(&mut self, semantics: &crate::Semantics, canvas: &mut skia_safe::Canvas) {
        let layout = semantics.layout(self.layout_node.unwrap());
        self.modifier.paint(&layout, canvas)
    }

    fn remove(&mut self, semantics: &mut crate::Semantics) {
        semantics.taffy.remove(self.layout_node.unwrap()).unwrap();
    }

    fn any(&self) -> &dyn std::any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
