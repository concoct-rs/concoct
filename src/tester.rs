use crate::{Composer, Semantics};
use accesskit::{Action, Node, NodeId};
use std::sync::Arc;

pub struct Tester {
    pub semantics: Semantics,
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

    pub fn get<'a>(
        &'a mut self,
        mut f: impl FnMut(NodeId, Arc<Node>) -> bool,
    ) -> Option<TestNode<'a>> {
        if self.should_recompose {
            Composer::recompose();
        } else {
            self.should_recompose = true;
        }

        Composer::with(|composer| composer.borrow_mut().semantics(&mut self.semantics));

        for (id, node) in &self.semantics.nodes {
            if f(*id, node.clone()) {
                return Some(TestNode { tester: self });
            }
        }

        None
    }
}

pub struct TestNode<'a> {
    tester: &'a mut Tester,
}

impl<'a> TestNode<'a> {
    pub fn click(&mut self) {
        for handler in self.tester.semantics.handlers.values_mut() {
            handler(Action::Default)
        }
    }
}
