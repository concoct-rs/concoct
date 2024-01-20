use crate::{Tree, View};

pub struct Empty;

impl View for Empty {
    fn into_tree(self) -> impl Tree {
        self
    }
}
