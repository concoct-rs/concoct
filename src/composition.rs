use crate::{
    AnyComposable, BuildContext, Composable, Inner, LocalContext, Node, TaskContext, BUILD_CONTEXT,
    TASK_CONTEXT,
};
use futures::pending;
use slotmap::DefaultKey;
use std::{any::Any, cell::RefCell, rc::Rc};
use tokio::{sync::mpsc, task::LocalSet};

/// A composition of composables.
pub struct Composition {
    build_cx: Rc<RefCell<BuildContext>>,
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

        let build_cx = Rc::new(RefCell::new(BuildContext::default()));
        BUILD_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(build_cx.clone()))
            .unwrap();

        let make_composable = Box::new(move || {
            let composable: Box<dyn AnyComposable> = Box::new(content());
            composable
        });
        let node = Node {
            make_composable,
            composable: None,

            hooks: Rc::default(),
        };
        let root = build_cx.borrow_mut().nodes.insert(node);

        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            build_cx,
            root,
            local_set: LocalSet::new(),
            task_cx: TaskContext { tx },
            rx,
        }
    }

    pub fn len(&self) -> usize {
        self.build_cx.borrow().nodes.len()
    }

    /// Build the initial composition.
    pub fn build(&mut self) {
        TASK_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.task_cx.clone()))
            .unwrap();

        BUILD_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.build_cx.clone()))
            .unwrap();

        let g = self.local_set.enter();
        let mut composable = {
            let mut build_cx = self.build_cx.borrow_mut();
            let node = &mut build_cx.nodes[self.root];
            let cx = LocalContext {
                inner: Rc::new(RefCell::new(Inner {
                    hooks: node.hooks.clone(),
                    idx: 0,
                })),
            };
            cx.enter();

            (node.make_composable)()
        };

        composable.any_build();
        drop(g);

        let mut build_cx = self.build_cx.borrow_mut();
        let node = &mut build_cx.nodes[self.root];
        node.composable = Some(composable);

        if let Some(children) = build_cx.children.get(self.root) {
            for child_key in children.clone() {
                let node = &mut build_cx.nodes[child_key];
                let cx = LocalContext {
                    inner: Rc::new(RefCell::new(Inner {
                        hooks: node.hooks.clone(),
                        idx: 0,
                    })),
                };
                cx.enter();

                let g = self.local_set.enter();
                let mut composable = {
                    let node = &mut build_cx.nodes[self.root];
                    let cx = LocalContext {
                        inner: Rc::new(RefCell::new(Inner {
                            hooks: node.hooks.clone(),
                            idx: 0,
                        })),
                    };
                    cx.enter();

                    (node.make_composable)()
                };
                drop(g);

                composable.any_build();

                let node = &mut build_cx.nodes[child_key];
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

        BUILD_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.build_cx.clone()))
            .unwrap();

        let g = self.local_set.enter();
        let mut composable = {
            let mut build_cx = self.build_cx.borrow_mut();
            let node = &mut build_cx.nodes[self.root];
            let cx = LocalContext {
                inner: Rc::new(RefCell::new(Inner {
                    hooks: node.hooks.clone(),
                    idx: 0,
                })),
            };
            cx.enter();

            (node.make_composable)()
        };
        composable.any_build();
        drop(g);

        let mut build_cx = self.build_cx.borrow_mut();
        let node = &mut build_cx.nodes[self.root];
        node.composable = Some(composable);

        if let Some(children) = build_cx.children.get(self.root) {
            for child_key in children.clone() {
                let node = &mut build_cx.nodes[child_key];

                let cx = LocalContext {
                    inner: Rc::new(RefCell::new(Inner {
                        hooks: node.hooks.clone(),
                        idx: 0,
                    })),
                };
                cx.enter();

                let g = self.local_set.enter();
                let mut composable = (node.make_composable)();
                composable.any_build();
                drop(g);

                let node = &mut build_cx.nodes[child_key];
                node.composable = Some(composable);
            }
        }

        TASK_CONTEXT.try_with(|cx| *cx.borrow_mut() = None).unwrap();
    }
}
