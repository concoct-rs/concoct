use accesskit::{Action, Node, NodeId, TreeUpdate};
use std::{collections::HashMap, fmt, mem, num::NonZeroU128, sync::Arc};
use taffy::{
    error::TaffyResult,
    layout::Cache,
    node::{Measurable, MeasureFunc},
    prelude::{AvailableSpace, Layout, Size},
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
    pub handlers: HashMap<NodeId, Box<dyn FnMut(Action)>>,
    pub taffy: Taffy,
    pub layout_children: Vec<Vec<LayoutNode>>,
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
        self.layout_children.push(Vec::new());
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
       children: &[LayoutNode]
    ) -> LayoutNode {
        let layout_id = self
            .taffy
            .new_with_children(style, children)
            .unwrap();
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

    pub fn tree_update(&mut self) -> TreeUpdate {
        mem::take(&mut self.tree_update)
    }
}

impl LayoutTree for Semantics {
    fn children(&self, node: LayoutNode) -> &[LayoutNode] {
        LayoutTree::children(&self.taffy, node)
    }

    fn child(&self, node: LayoutNode, index: usize) -> LayoutNode {
        LayoutTree::child(&self.taffy, node, index)
    }

    fn parent(&self, node: LayoutNode) -> Option<LayoutNode> {
        LayoutTree::parent(&self.taffy, node)
    }

    fn style(&self, node: LayoutNode) -> &Style {
        LayoutTree::style(&self.taffy, node)
    }

    fn layout(&self, node: LayoutNode) -> &Layout {
        LayoutTree::layout(&self.taffy, node)
    }

    fn layout_mut(&mut self, node: LayoutNode) -> &mut Layout {
        LayoutTree::layout_mut(&mut self.taffy, node)
    }

    fn mark_dirty(&mut self, node: LayoutNode) -> TaffyResult<()> {
        LayoutTree::mark_dirty(&mut self.taffy, node)
    }

    fn measure_node(
        &self,
        node: LayoutNode,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        LayoutTree::measure_node(&self.taffy, node, known_dimensions, available_space)
    }

    fn needs_measure(&self, node: LayoutNode) -> bool {
        LayoutTree::needs_measure(&self.taffy, node)
    }

    fn cache_mut(&mut self, node: LayoutNode, index: usize) -> &mut Option<Cache> {
        LayoutTree::cache_mut(&mut self.taffy, node, index)
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
                    &Self {
                        children: &node.children,
                        semantics: self.semantics,
                    },
                );
            }

            debug_struct.finish()?;
        }

        Ok(())
    }
}
