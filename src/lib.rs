use std::{collections::HashMap, num::NonZeroU128};

pub mod view;
use accesskit::{NodeBuilder, NodeClassSet, NodeId, Role, TreeUpdate};
use slotmap::{DefaultKey, SlotMap};
use taffy::{
    prelude::{Node, Size},
    style::{FlexDirection, Style},
    style_helpers::{points, TaffyMaxContent},
    Taffy,
};
pub use view::View;

pub struct SemanticsContext<'a> {
    next_id: &'a mut NonZeroU128,
    unused_ids: &'a mut Vec<NodeId>,
    tree_update: &'a mut TreeUpdate,
    node_children: &'a mut Vec<NodeId>,
}

impl SemanticsContext<'_> {
    pub fn node_id(&mut self) -> NodeId {
        if let Some(id) = self.unused_ids.pop() {
            id
        } else {
            let id = *self.next_id;
            *self.next_id = self.next_id.checked_add(1).unwrap();
            NodeId(id)
        }
    }
}

pub struct Context {
    next_id: NonZeroU128,
    unused_ids: Vec<NodeId>,
    handlers: HashMap<usize, Box<dyn FnMut()>>,
    taffy: Taffy,
    root: Option<DefaultKey>,
    children: Vec<Node>,
    widgets: SlotMap<DefaultKey, Box<dyn Widget>>,
    updated: Vec<DefaultKey>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            next_id: unsafe { NonZeroU128::new_unchecked(2) },
            unused_ids: Vec::new(),
            handlers: HashMap::new(),
            taffy: Taffy::new(),
            root: None,
            children: Vec::new(),
            widgets: SlotMap::new(),
            updated: Vec::new(),
        }
    }

    pub fn handle(&mut self, id: usize) {
        self.handlers.get_mut(&id).unwrap()();
    }

    pub fn layout(&mut self) {
        let root = if let Some(root) = self.root {
            self.taffy.set_children(root, &self.children).unwrap();
            root
        } else {
            let style = Style {
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: points(800.0),
                    height: points(600.0),
                },
                ..Default::default()
            };
            let root = self.taffy.new_with_children(style, &self.children).unwrap();
            self.root = Some(root);
            root
        };

        self.taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    }

    pub fn tree_update(&mut self) -> TreeUpdate {
        let mut tree_update = TreeUpdate::default();
        let mut node_children = Vec::new();

        for id in std::mem::take(&mut self.updated) {
            let widget = self.widgets.get_mut(id).unwrap();
            widget.semantics(&mut SemanticsContext {
                next_id: &mut self.next_id,
                unused_ids: &mut self.unused_ids,
                tree_update: &mut tree_update,
                node_children: &mut node_children,
            });
        }

        const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
        let mut builder = NodeBuilder::new(Role::Window);
        builder.set_children(node_children);
        builder.set_name("WINDOW_TITLE");
        let node = builder.build(&mut NodeClassSet::lock_global());
        tree_update.nodes.push((WINDOW_ID, node));

        tree_update
    }
}

pub trait Widget {
    fn semantics(&mut self, cx: &mut SemanticsContext);
}

#[cfg(test)]
mod tests {
    use crate::{view::Text, Context, View};

    #[test]
    fn f() {
        let mut cx = Context::new();
        let mut text = Text::new("Test");
        text.view(&mut cx);

        cx.layout();

        dbg!(cx.tree_update());
    }
}
