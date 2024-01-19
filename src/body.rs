use crate::{Node, Scope, Tree, View};

pub trait Body {
    fn tree(self) -> impl Tree;
}

pub struct Empty;

impl Body for Empty {
    fn tree(self) -> impl Tree {
        self
    }
}

impl<V: View> Body for V {
    fn tree(self) -> impl Tree {
        Node {
            view: self,
            body: None,
            builder: |me: &'static V| me.body().tree(),
            scope: Scope::default(),
        }
    }
}

impl<V1: Body, V2: Body> Body for (V1, V2) {
    fn tree(self) -> impl Tree {
        (self.0.tree(), self.1.tree())
    }
}
