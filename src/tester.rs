use std::sync::Arc;

use accesskit::Node;

use crate::{Composer, Semantics};

pub struct Tester {
    semantics: Semantics,
    should_recompose: bool,
}

impl Tester {
    pub fn new(f: impl FnOnce()) -> Self {
        f();
        Self {
            semantics: Semantics::default(),
            should_recompose: false,
        }
    }

    pub fn nodes(&mut self) -> impl Iterator<Item = &Arc<Node>> {
        if self.should_recompose {
            Composer::recompose();
        } else {
            self.should_recompose = true;
        }

        Composer::with(|composer| composer.borrow_mut().semantics(&mut self.semantics));

        self.semantics.nodes.values()
    }
}
