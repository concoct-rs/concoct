use accesskit::{Node, NodeBuilder, NodeClassSet, NodeId, Role, Tree, TreeUpdate};
use std::{mem, num::NonZeroU128};

pub struct Context {
    next_id: NonZeroU128,
    unused_ids: Vec<NodeId>,
    nodes: Vec<(NodeId, Node)>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            next_id: NonZeroU128::MIN,
            unused_ids: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn node_id(&mut self) -> NodeId {
        if let Some(node_id) = self.unused_ids.pop() {
            node_id
        } else {
            let id = self.next_id;
            self.next_id = self.next_id.checked_add(1).unwrap();
            NodeId(id)
        }
    }
}

pub trait Semantics {
    fn build(&mut self, cx: &mut Context) -> NodeBuilder;

    fn rebuild(&mut self, cx: &mut Context, old: &mut Self) -> Option<NodeBuilder>;
}

pub struct Text {
    string: String,
}

impl Text {
    pub fn new(string: impl Into<String>) -> Self {
        Self {
            string: string.into(),
        }
    }
}

impl Semantics for Text {
    fn build(&mut self, cx: &mut Context) -> NodeBuilder {
        NodeBuilder::new(Role::StaticText)
    }

    fn rebuild(&mut self, cx: &mut Context, old: &mut Self) -> Option<NodeBuilder> {
        if self.string != old.string {
            Some(self.build(cx))
        } else {
            None
        }
    }
}

pub struct Child<T> {
    node_id: Option<NodeId>,
    semantics: T,
}

impl<T> Child<T> {
    pub fn new(semantics: T) -> Self {
        Self {
            node_id: None,
            semantics,
        }
    }
}

pub struct Row<T> {
    children: T,
}

impl<T> Row<T> {
    pub fn new(children: T) -> Self {
        Self { children }
    }
}

impl<A, B> Semantics for Row<(Child<A>, Child<B>)>
where
    A: Semantics,
    B: Semantics,
{
    fn build(&mut self, cx: &mut Context) -> NodeBuilder {
        let mut row_builder = NodeBuilder::new(Role::Row);

        let builder = self.children.0.semantics.build(cx);
        let node = builder.build(&mut NodeClassSet::lock_global());

        let node_id = cx.node_id();
        self.children.0.node_id = Some(node_id);

        cx.nodes.push((node_id, node));
        row_builder.push_child(node_id);

        row_builder
    }

    fn rebuild(&mut self, cx: &mut Context, old: &mut Self) -> Option<NodeBuilder> {
        if let Some(builder) = self
            .children
            .0
            .semantics
            .rebuild(cx, &mut old.children.0.semantics)
        {
            let node = builder.build(&mut NodeClassSet::lock_global());

            let node_id = old.children.0.node_id.unwrap();
            self.children.0.node_id = Some(node_id);

            cx.nodes.push((node_id, node));
        }

        None
    }
}

pub struct Composer {
    context: Context,
    is_tree_changed: bool,
}

impl Composer {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            is_tree_changed: true,
        }
    }

    pub fn tree_update(&mut self, mut semantics: impl Semantics) -> TreeUpdate {
        let node_id = self.context.node_id();
        let tree = if self.is_tree_changed {
            Some(Tree::new(node_id))
        } else {
            self.is_tree_changed = false;
            None
        };

        let builder = semantics.build(&mut self.context);
        let node = builder.build(&mut NodeClassSet::lock_global());

        let mut nodes = mem::take(&mut self.context.nodes);
        nodes.push((node_id, node));

        TreeUpdate {
            nodes,
            tree,
            focus: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Child, Composer, Context, Row, Semantics, Text};

    #[test]
    fn it_works() {
        let mut cx = Context::new();

        let mut text = Text::new("old");
        text.build(&mut cx);

        let mut new_text = Text::new("old");
        assert!(new_text.rebuild(&mut cx, &mut text).is_none());

        let mut new_text = Text::new("new");
        assert!(new_text.rebuild(&mut cx, &mut text).is_some());
    }

    #[test]
    fn container() {
        let mut composer = Composer::new();
        let semantics = Row::new((Child::new(Text::new("A")), Child::new(Text::new("B"))));

        dbg!(composer.tree_update(semantics));
    }
}
