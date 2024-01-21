use crate::{Tree, View};
use std::any::Any;

/// Empty view.
pub struct Empty;

impl View for Empty {
    fn into_tree(self) -> impl Tree {
        self
    }
}

impl Tree for Empty {
    unsafe fn build(&mut self) {}

    unsafe fn rebuild(&mut self, _last: &mut dyn Any, _is_changed: bool) {}

    unsafe fn remove(&mut self) {}
}
