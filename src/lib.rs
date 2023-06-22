use accesskit::{Node, NodeBuilder, NodeClassSet, NodeId, Role, TreeUpdate};
use std::{mem, num::NonZeroU128};

pub struct Context {
    next_id: NonZeroU128,
    nodes: Vec<(NodeId, Node)>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            next_id: NonZeroU128::MIN,
            nodes: Vec::new(),
        }
    }

    pub fn node_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).unwrap();

        NodeId(id)
    }

    pub fn tree_update(&mut self) -> TreeUpdate {
        let mut tree_update = TreeUpdate::default();
        tree_update.nodes = mem::take(&mut self.nodes);
        tree_update
    }
}

pub trait Semantics {
    fn build(&mut self, cx: &mut Context);

    fn rebuild(&mut self, cx: &mut Context, old: Self);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    string: String,
    node_id: Option<NodeId>,
}

impl Text {
    pub fn new(string: impl Into<String>) -> Self {
        Self {
            string: string.into(),
            node_id: None,
        }
    }
}

impl Semantics for Text {
    fn build(&mut self, cx: &mut Context) {
        let mut builder = NodeBuilder::new(Role::StaticText);
        builder.set_value(self.string.clone());
        let node = builder.build(&mut NodeClassSet::lock_global());

        let node_id = cx.node_id();
        self.node_id = Some(node_id);

        cx.nodes.push((node_id, node));
    }

    fn rebuild(&mut self, cx: &mut Context, old: Self) {
        if *self != old {
            let mut builder = NodeBuilder::new(Role::StaticText);
            builder.set_value(self.string.clone());
            let node = builder.build(&mut NodeClassSet::lock_global());

            let node_id = old.node_id.unwrap();
            self.node_id = Some(node_id);

            cx.nodes.push((node_id, node));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Context, Semantics, Text};

    #[test]
    fn f() {
        let mut text = Text::new("old");
        let mut cx = Context::new();

        text.build(&mut cx);
        dbg!(cx.tree_update());

        let mut new_text = Text::new("new");
        new_text.rebuild(&mut cx, text);
        dbg!(cx.tree_update());
    }
}
