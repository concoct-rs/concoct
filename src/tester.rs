use crate::{Composer, Event, Semantics};
use accesskit::{Action, Node, NodeId};

pub trait Matcher {
    fn is_match(&mut self, node_id: NodeId, node: &Node) -> bool;
}

impl<F> Matcher for F
where
    F: FnMut(NodeId, &Node) -> bool,
{
    fn is_match(&mut self, node_id: NodeId, node: &Node) -> bool {
        self(node_id, node)
    }
}

pub struct Tester {
    pub semantics: Semantics,
    should_recompose: bool,
}

impl Tester {
    pub fn new(f: impl FnOnce()) -> Self {
        f();

        let mut me = Self {
            semantics: Semantics::default(),
            should_recompose: false,
        };

        Composer::with(|composer| {
            let mut cx = composer.borrow_mut();
            cx.layout(&mut me.semantics);

            me.semantics.children = vec![Vec::new()];
            cx.semantics(&mut me.semantics);
        });

        me
    }

    pub fn get(&mut self, mut matcher: impl Matcher) -> Option<TestNode> {
        if self.should_recompose {
            Composer::recompose(&mut self.semantics);
        } else {
            self.should_recompose = true;
        }

        Composer::with(|composer| {
            let mut cx = composer.borrow_mut();
            cx.layout(&mut self.semantics);

            self.semantics.children = vec![Vec::new()];
            cx.semantics(&mut self.semantics);
        });

        for (id, node) in &self.semantics.nodes {
            if matcher.is_match(*id, node.as_ref()) {
                return Some(TestNode { tester: self });
            }
        }

        None
    }

    pub fn get_text(&mut self, text: impl AsRef<str>) -> Option<TestNode<'_>> {
        self.get(|_node_id, node: &Node| node.value.as_deref() == Some(text.as_ref()))
    }
}

pub struct TestNode<'a> {
    tester: &'a mut Tester,
}

impl<'a> TestNode<'a> {
    pub fn click(&mut self) {
        for (node_id, handler) in self.tester.semantics.handlers.iter_mut() {
            let node = self.tester.semantics.nodes.get(&node_id).unwrap();
            handler(node, Event::Action(Action::Default))
        }
    }
}
