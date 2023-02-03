use crate::Event;
use accesskit::{Node, NodeId, TreeUpdate};
use skia_safe::Point;
use std::{collections::HashMap, fmt, mem, num::NonZeroU128, sync::Arc};
use taffy::{
    node::{Measurable, MeasureFunc},
    prelude::Layout,
    style::Style,
    tree::LayoutTree,
    Taffy,
};

pub type LayoutNode = taffy::prelude::Node;

pub struct Semantics {
    pub nodes: HashMap<NodeId, Arc<Node>>,
    pub children: Vec<Vec<NodeId>>,
    high_water_mark: NonZeroU128,
    unused_ids: Vec<NodeId>,
    tree_update: TreeUpdate,
    pub handlers: HashMap<NodeId, Box<dyn FnMut(Event)>>,
    pub taffy: Taffy,
    pub layout_children: Vec<Vec<LayoutNode>>,
    pub points: Vec<Point>,
}

impl Default for Semantics {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            children: vec![Vec::new()],
            high_water_mark: NonZeroU128::new(1).unwrap(),
            unused_ids: Vec::new(),
            tree_update: TreeUpdate::default(),
            handlers: HashMap::new(),
            taffy: Taffy::new(),
            layout_children: vec![Vec::new()],
            points: Vec::new(),
        }
    }
}

impl Semantics {
    pub fn insert(&mut self, node: Node) -> NodeId {
        let id = if let Some(id) = self.unused_ids.pop() {
            id
        } else {
            let id = NodeId(self.high_water_mark);
            self.high_water_mark = self.high_water_mark.checked_add(1).unwrap();
            id
        };

        let node = Arc::new(node);

        self.nodes.insert(id, node.clone());
        self.children.last_mut().unwrap().push(id);
        self.tree_update.nodes.push((id, node));

        id
    }

    pub fn update(&mut self, id: NodeId, node: Node) {
        self.children.last_mut().unwrap().push(id);

        let last_node = self.nodes.get_mut(&id).unwrap();
        if &node != &**last_node {
            let node = Arc::new(node);
            *last_node = node.clone();
            self.tree_update.nodes.push((id, node));
        }
    }

    pub fn start_group(&mut self) {
        self.children.push(Vec::new());
    }

    pub fn end_group(&mut self) -> NodeId {
        let children = self.children.pop().unwrap();

        let node = Node {
            children,
            ..Node::default()
        };
        self.insert(node)
    }

    pub fn end_group_with_node(&mut self, mut node: Node, merge: bool) -> NodeId {
        let children = self.children.pop().unwrap();

        if merge {
            for id in &children {
                let child = self.remove(*id).unwrap();
                if let Some(value) = &child.value {
                    if node.value.is_none() {
                        node.value = Some(value.clone());
                    }
                }
            }
        } else {
            node.children = children;
        }

        self.insert(node)
    }

    pub fn end_group_update(&mut self, id: NodeId) {
        let children = self.children.pop().unwrap();
        let node = Node {
            children,
            ..Node::default()
        };

        self.update(id, node);
    }

    pub fn remove(&mut self, id: NodeId) -> Option<Arc<Node>> {
        if let Some(node) = self.nodes.remove(&id) {
            self.unused_ids.push(id);

            Some(node)
        } else {
            None
        }
    }

    pub fn insert_layout_with_children(
        &mut self,
        style: Style,
        children: &[LayoutNode],
    ) -> LayoutNode {
        let layout_id = self.taffy.new_with_children(style, children).unwrap();
        self.layout_children.last_mut().unwrap().push(layout_id);

        layout_id
    }

    pub fn insert_layout_with_measure(
        &mut self,
        style: Style,
        measure: impl Measurable + 'static,
    ) -> LayoutNode {
        let layout_id = self
            .taffy
            .new_leaf_with_measure(style, MeasureFunc::Boxed(Box::new(measure)))
            .unwrap();
        self.layout_children.last_mut().unwrap().push(layout_id);

        layout_id
    }

    pub fn layout(&self, layout_id: LayoutNode) -> Layout {
        let parent_point = self.points.last().unwrap();

        let mut layout = self.taffy.layout(layout_id).unwrap().clone();
        layout.location.x += parent_point.x;
        layout.location.y += parent_point.y;

        layout
    }

    pub fn tree_update(&mut self) -> TreeUpdate {
        mem::take(&mut self.tree_update)
    }
}

impl fmt::Debug for Semantics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Semantics")
            .field(
                "nodes",
                &Wrap {
                    children: &self.children.last().unwrap(),
                    semantics: self,
                },
            )
            .finish()
    }
}

struct Wrap<'a> {
    children: &'a [NodeId],
    semantics: &'a Semantics,
}

impl fmt::Debug for Wrap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for node_id in self.children {
            let node = &self.semantics.nodes[node_id];

            let mut debug_struct = f.debug_struct("Node");
            debug_struct.field("id", &node_id.0);
            debug_struct.field("role", &node.role);

            if let Some(value) = &node.value {
                debug_struct.field("value", value);
            }

            if !node.children.is_empty() {
                debug_struct.field(
                    "children",
                    &[Self {
                        children: &node.children,
                        semantics: self.semantics,
                    }],
                );
            }

            debug_struct.finish()?;
        }

        Ok(())
    }
}
