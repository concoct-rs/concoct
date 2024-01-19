use crate::Tree;

pub trait Body {
    fn tree(self) -> impl Tree;
}

pub struct Empty;

impl Body for Empty {
    fn tree(self) -> impl Tree {
        self
    }
}
