use accesskit::{Node, NodeBuilder, NodeClassSet, Role};

pub struct Context {
    nodes: Vec<Node>,
}

pub trait Semantics {
    fn build(&mut self, cx: &mut Context);

    fn rebuild(&mut self, cx: &mut Context, old: Self);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    string: String,
}

impl Semantics for Text {
    fn build(&mut self, cx: &mut Context) {
        let mut builder = NodeBuilder::new(Role::StaticText);
        builder.set_value(self.string.clone());
        let node = builder.build(&mut NodeClassSet::lock_global());
        cx.nodes.push(node);
    }

    fn rebuild(&mut self, cx: &mut Context, old: Self) {
        if *self != old {
            self.build(cx)
        }
    }
}
