use crate::{Node, Scope, Tree, View};

pub trait Body: 'static {
    fn into_tree(self) -> impl Tree;
}

pub struct Empty;

impl Body for Empty {
    fn into_tree(self) -> impl Tree {
        self
    }
}

impl<V: View> Body for V {
    fn into_tree(self) -> impl Tree {
        Node {
            view: self,
            body: None,
            builder: |me: &'static V| me.body().into_tree(),
            scope: Scope::default(),
            key: None,
        }
    }
}

impl<V1: Body, V2: Body> Body for (V1, V2) {
    fn into_tree(self) -> impl Tree {
        (self.0.into_tree(), self.1.into_tree())
    }
}
