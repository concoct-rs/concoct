use crate::{
    composable::IntoComposable, AnyComposable, BuildContext, Composable, Inner, LocalContext, Node,
    TaskContext, BUILD_CONTEXT, TASK_CONTEXT,
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
        C: IntoComposable + 'static,
    {
        let local_set = LocalSet::new();
        local_set.enter();

        {
            let build_cx = Rc::new(RefCell::new(BuildContext::default()));
            BUILD_CONTEXT
                .try_with(|cx| *cx.borrow_mut() = Some(build_cx.clone()))
                .unwrap();
        }

        let make_composable = Box::new(move || {
            let composable: Box<dyn AnyComposable> = Box::new(content().into_composer());
            composable
        });

        let build_cx = Rc::new(RefCell::new(BuildContext::default()));
        let node = Node {
            make_composable,
            composable: None,
            hooks: Rc::default(),
        };
        let root = build_cx
            .borrow_mut()
            .nodes
            .insert(Rc::new(RefCell::new(node)));
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

    pub fn compose(&mut self, key: DefaultKey) {
        TASK_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.task_cx.clone()))
            .unwrap();

        self.build_cx.borrow_mut().parent_key = key;
        BUILD_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.build_cx.clone()))
            .unwrap();

        let g = self.local_set.enter();

        {
            let cx = self.build_cx.borrow_mut();
            let node = cx.nodes[key].borrow_mut();
            let cx = LocalContext {
                inner: Rc::new(RefCell::new(Inner {
                    hooks: node.hooks.clone(),
                    idx: 0,
                })),
            };
            cx.enter();
        }

        let composable = {
            let node = {
                let build_cx = self.build_cx.borrow_mut();
                build_cx.nodes[key].clone()
            };
            let mut node = node.borrow_mut();
            let new_composable = (node.make_composable)();

            if let Some(ref composable) = node.composable {
                if new_composable.any_eq(composable.borrow().as_any()) {
                    return;
                } else {
                    *composable.borrow_mut() = new_composable;
                    node.composable.as_ref().unwrap().clone()
                }
            } else {
                node.composable = Some(Rc::new(RefCell::new(new_composable)));
                node.composable.as_ref().unwrap().clone()
            }
        };
        composable.borrow_mut().any_build();

        let children = self.build_cx.borrow().children.get(key).cloned();
        if let Some(children) = children {
            for child_key in children {
                let g = self.local_set.enter();

                {
                    let mut build_cx = self.build_cx.borrow_mut();
                    build_cx.parent_key = child_key;
                    let node = build_cx.nodes[child_key].borrow_mut();
                    let cx = LocalContext {
                        inner: Rc::new(RefCell::new(Inner {
                            hooks: node.hooks.clone(),
                            idx: 0,
                        })),
                    };
                    cx.enter();
                }

                let composable = {
                    let node = {
                        let build_cx = self.build_cx.borrow_mut();
                        build_cx.nodes[child_key].clone()
                    };
                    let mut node = node.borrow_mut();

                    let new_composable = (node.make_composable)();

                    if let Some(ref composable) = node.composable {
                        if new_composable.any_eq(composable.borrow().as_any()) {
                            continue;
                        } else {
                            *composable.borrow_mut() = new_composable;
                            node.composable.as_ref().unwrap().clone()
                        }
                    } else {
                        node.composable = Some(Rc::new(RefCell::new(new_composable)));
                        node.composable.as_ref().unwrap().clone()
                    }
                };

                drop(g);
                composable.borrow_mut().any_build();
            }
        }

        drop(g);
        TASK_CONTEXT.try_with(|cx| *cx.borrow_mut() = None).unwrap();
    }

    /// Build the initial composition.
    pub fn build(&mut self) {
        self.compose(self.root);
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

        self.compose(self.root);
    }
}
