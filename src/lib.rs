use accesskit::{NodeBuilder, NodeId, Role};
use std::num::NonZeroU128;

pub struct Context {
    next_id: NonZeroU128,
    unused_ids: Vec<NodeId>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            next_id: NonZeroU128::MIN,
            unused_ids: Vec::new(),
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
    fn build(&mut self) -> NodeBuilder;

    fn rebuild(&mut self, old: &mut Self) -> Option<NodeBuilder>;
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
    fn build(&mut self) -> NodeBuilder {
        NodeBuilder::new(Role::StaticText)
    }

    fn rebuild(&mut self, old: &mut Self) -> Option<NodeBuilder> {
        if self.string != old.string {
            Some(self.build())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Semantics, Text};

    #[test]
    fn it_works() {
        let mut text = Text::new("old");
        text.build();

        let mut new_text = Text::new("old");
        assert!(new_text.rebuild(&mut text).is_none());

        let mut new_text = Text::new("new");
        assert!(new_text.rebuild(&mut text).is_some());
    }
}
