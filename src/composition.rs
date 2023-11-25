use crate::{
    AnyComposable, BuildContext, Composable, Inner, LocalContext, Node, TaskContext, TASK_CONTEXT,
};
use futures::pending;
use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{any::Any, cell::RefCell, rc::Rc};
use tokio::{sync::mpsc, task::LocalSet};

/// A composition of composables.
pub struct Composition {
    nodes: SlotMap<DefaultKey, Node>,
    children: SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
    root: DefaultKey,
    local_set: LocalSet,
    task_cx: TaskContext,
    rx: mpsc::UnboundedReceiver<Box<dyn Any>>,
}

impl Composition {
    /// Create a new composition from it's root composable function.
    pub fn new<C>(content: fn() -> C) -> Self
    where
        C: Composable + 'static,
    {
        let local_set = LocalSet::new();
        local_set.enter();

        let mut composables = SlotMap::new();
        let make_composable = Box::new(move || {
            let composable: Box<dyn AnyComposable> = Box::new(content());
            composable
        });
        let node = Node {
            make_composable,
            composable: None,

            hooks: Rc::default(),
        };
        let root = composables.insert(node);

        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            nodes: composables,
            children: SparseSecondaryMap::new(),
            root,
            local_set: LocalSet::new(),
            task_cx: TaskContext { tx },
            rx,
        }
    }

    /// Build the initial composition.
    pub fn build(&mut self) {
        TASK_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.task_cx.clone()))
            .unwrap();

        let node = &mut self.nodes[self.root];
        let cx = LocalContext {
            inner: Rc::new(RefCell::new(Inner {
                hooks: node.hooks.clone(),
                idx: 0,
            })),
        };
        cx.enter();

        let g = self.local_set.enter();
        let mut composable = (node.make_composable)();
        drop(g);

        let mut build_cx = BuildContext {
            parent_key: self.root,
            nodes: &mut self.nodes,
            children: &mut self.children,
        };
        let _state = composable.any_build(&mut build_cx);

        let node = &mut self.nodes[self.root];
        node.composable = Some(composable);

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

                let g = self.local_set.enter();
                let mut composable = (node.make_composable)();
                drop(g);

                let mut build_cx = BuildContext {
                    parent_key: child_key,
                    nodes: &mut self.nodes,
                    children: &mut self.children,
                };
                let _state = composable.any_build(&mut build_cx);

                let node = &mut self.nodes[child_key];
                node.composable = Some(composable);
            }
        }

        TASK_CONTEXT.try_with(|cx| *cx.borrow_mut() = None).unwrap();
    }

    /// Rebuild the composition from it's previous state.
    pub async fn rebuild(&mut self) {
        TASK_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.task_cx.clone()))
            .unwrap();

        loop {
            let fut = async {
                self.rx.recv().await;
            };

            if futures::poll!(Box::pin(fut)).is_ready() {
                break;
            }

            let fut = async {
                let fut = &mut self.local_set;
                fut.await;
            };
            if futures::poll!(Box::pin(fut)).is_pending() {
                pending!()
            } else {
                break;
            }
        }

        let node = &mut self.nodes[self.root];
        let cx = LocalContext {
            inner: Rc::new(RefCell::new(Inner {
                hooks: node.hooks.clone(),
                idx: 0,
            })),
        };
        cx.enter();

        let g = self.local_set.enter();
        let mut composable = (node.make_composable)();
        drop(g);

        let mut build_cx = BuildContext {
            parent_key: self.root,
            nodes: &mut self.nodes,
            children: &mut self.children,
        };
        composable.any_build(&mut build_cx);

        let node = &mut self.nodes[self.root];

        node.composable = Some(composable);

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

                let g = self.local_set.enter();
                let mut composable = (node.make_composable)();
                drop(g);

                let mut build_cx = BuildContext {
                    parent_key: child_key,
                    nodes: &mut self.nodes,
                    children: &mut self.children,
                };
                composable.any_build(&mut build_cx);

                let node = &mut self.nodes[child_key];
                node.composable = Some(composable);
            }
        }

        TASK_CONTEXT.try_with(|cx| *cx.borrow_mut() = None).unwrap();
    }
}
