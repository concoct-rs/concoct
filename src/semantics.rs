use accesskit::{Node, NodeId, TreeUpdate};
use std::{collections::HashMap, fmt, mem, num::NonZeroU128, sync::Arc};

pub struct Semantics {
    pub nodes: HashMap<NodeId, Arc<Node>>,
    children: Vec<Vec<NodeId>>,
    high_water_mark: NonZeroU128,
    unused_ids: Vec<NodeId>,
    tree_update: TreeUpdate,
}

impl Default for Semantics {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            children: vec![Vec::new()],
            high_water_mark: NonZeroU128::new(1).unwrap(),
            unused_ids: Default::default(),
            tree_update: TreeUpdate::default(),
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

    pub fn end_group_with_node(&mut self, mut node: Node) -> NodeId {
        node.children = self.children.pop().unwrap();

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
