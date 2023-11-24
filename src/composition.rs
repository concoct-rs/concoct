use crate::{AnyComposable, BuildContext, Composable, Inner, LocalContext, Node};
use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{cell::RefCell, rc::Rc};

pub struct Composition {
    nodes: SlotMap<DefaultKey, Node>,
    children: SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
    root: DefaultKey,
}

impl Composition {
    pub fn new<C>(content: fn() -> C) -> Self
    where
        C: Composable + 'static,
    {
        let mut composables = SlotMap::new();
        let make_composable = Box::new(move || {
            let composable: Box<dyn AnyComposable> = Box::new(content());
            composable
        });
        let node = Node {
            make_composable,
            composable: None,
            state: None,
            hooks: Rc::default(),
        };
        let root = composables.insert(node);

        Self {
            nodes: composables,
            children: SparseSecondaryMap::new(),
            root,
        }
    }

    pub fn build(&mut self) {
        let node = &mut self.nodes[self.root];
        let cx = LocalContext {
            inner: Rc::new(RefCell::new(Inner {
                hooks: node.hooks.clone(),
                idx: 0,
            })),
        };
        cx.enter();
        let mut composable = (node.make_composable)();

        let mut build_cx = BuildContext {
            parent_key: self.root,
            nodes: &mut self.nodes,
            children: &mut self.children,
        };
        let state = composable.any_build(&mut build_cx);

        let node = &mut self.nodes[self.root];
        node.composable = Some(composable);
        node.state = Some(state);

        if let Some(children) = self.children.get(self.root) {
            for child_key in children.clone() {
                let node = &mut self.nodes[child_key];
                let cx = LocalContext {
                    inner: Rc::new(RefCell::new(Inner {
                        hooks: node.hooks.clone(),
                        idx: 0,
                    })),
                };
                cx.enter();
                let mut composable = (node.make_composable)();

                let mut build_cx = BuildContext {
                    parent_key: child_key,
                    nodes: &mut self.nodes,
                    children: &mut self.children,
                };
                let state = composable.any_build(&mut build_cx);

                let node = &mut self.nodes[child_key];
                node.composable = Some(composable);
                node.state = Some(state);
            }
        }
    }

    pub async fn rebuild(&mut self) {
        let node = &mut self.nodes[self.root];
        let cx = LocalContext {
            inner: Rc::new(RefCell::new(Inner {
                hooks: node.hooks.clone(),
                idx: 0,
            })),
        };
        cx.enter();

        let mut composable = (node.make_composable)();
        let state = node.state.as_mut().unwrap();
        composable.any_rebuild(&mut **state);
        node.composable = Some(composable);
    }
}
