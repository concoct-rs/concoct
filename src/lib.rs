use std::num::NonZeroU128;
use accesskit::{Node, NodeBuilder, NodeClassSet, NodeId, Role};

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
    fn is_changed(&self, old: &Self) -> bool;

    fn build(&mut self) -> NodeBuilder;

    fn modify<F>(self, f: F) -> ModifySemantics<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut NodeBuilder),
    {
        ModifySemantics { semantics: self, f }
    }

    fn build_node(&mut self) -> Node {
        self.build().build(&mut NodeClassSet::lock_global())
    }
}

pub struct Text {
    string: String,
}

impl Semantics for Text {
    fn is_changed(&self, old: &Self) -> bool {
        self.string != old.string
    }

    fn build(&mut self) -> NodeBuilder {
        let mut builder = NodeBuilder::new(Role::StaticText);
        builder.set_value(self.string.clone());
        builder
    }
}

pub struct ModifySemantics<T, F> {
    semantics: T,
    f: F,
}

impl<T, F> Semantics for ModifySemantics<T, F>
where
    T: Semantics,
    F: FnMut(&mut NodeBuilder),
{
    fn is_changed(&self, old: &Self) -> bool {
        self.semantics.is_changed(&old.semantics)
    }

    fn build(&mut self) -> NodeBuilder {
        let mut builder = self.semantics.build();
        (self.f)(&mut builder);
        builder
    }
}
