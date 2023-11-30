use crate::{
    AnyView, BuildContext, IntoView, LocalContext, Node, Platform, Scope, TaskContext,
    BUILD_CONTEXT, GLOBAL_CONTEXT, TASK_CONTEXT,
};
use futures::channel::mpsc;
use futures::executor::LocalPool;
use futures::StreamExt;
use slotmap::DefaultKey;
use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

/// A composition of views.
pub struct Tree {
    build_cx: Rc<RefCell<BuildContext>>,
    root: DefaultKey,
    task_cx: TaskContext,
    rx: mpsc::UnboundedReceiver<Box<dyn Any>>,
}

impl Tree {
    /// Create a new composition from it's root view function.
    pub fn new(platform: impl Platform + 'static, content: impl IntoView) -> Self {
        let local_set = LocalPool::new();
        let (tx, rx) = mpsc::unbounded();
        let task_cx = TaskContext {
            tx,
            local_pool: Rc::new(RefCell::new(local_set)),
        };

        let build_cx = Rc::new(RefCell::new(BuildContext::new(platform)));
        BUILD_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(build_cx.clone()))
            .unwrap();

        let mut content = Some(content);
        let make_view = Box::new(move || {
            let view: Box<dyn AnyView> = Box::new(content.take().unwrap().into_view());
            view
        });

        let node = Node {
            make_view,
            view: None,
            hooks: Rc::default(),
            contexts: HashMap::new(),
        };
        let root = build_cx
            .borrow_mut()
            .nodes
            .insert(Rc::new(RefCell::new(node)));

        Self {
            build_cx,
            root,
            task_cx,
            rx,
        }
    }

    pub fn len(&self) -> usize {
        self.build_cx.borrow().nodes.len()
    }

    // TODO switch from this recursive method
    pub fn compose(&mut self, key: DefaultKey) {
        let children = {
            TASK_CONTEXT
                .try_with(|cx| *cx.borrow_mut() = Some(self.task_cx.clone()))
                .unwrap();

            BUILD_CONTEXT
                .try_with(|cx| *cx.borrow_mut() = Some(self.build_cx.clone()))
                .unwrap();

            {
                let mut cx = self.build_cx.borrow_mut();
                let contexts = if let Some(parent_node) = cx.nodes.get(cx.parent_key) {
                    parent_node.borrow().contexts.clone()
                } else {
                    HashMap::new()
                };

                cx.parent_key = key;
                let node = cx.nodes[key].borrow_mut();

                let cx = LocalContext {
                    scope: Rc::new(RefCell::new(Scope {
                        hooks: node.hooks.clone(),
                        idx: 0,
                        contexts,
                    })),
                };
                cx.enter();
            }

            let view = {
                let node = {
                    let build_cx = self.build_cx.borrow_mut();
                    build_cx.nodes[key].clone()
                };
                let mut node = node.borrow_mut();
                let new_view = (node.make_view)();

                if let Some(ref view) = node.view {
                    let mut is_dirty = false;
                    if new_view.any_eq(view.borrow().as_any()) {
                        if let Some(tracked) = self.build_cx.borrow_mut().tracked.get(key) {
                            for tracked_key in tracked {
                                if GLOBAL_CONTEXT
                                    .try_with(|cx| cx.borrow().dirty.contains(tracked_key))
                                    .unwrap()
                                {
                                    is_dirty = true;
                                    break;
                                }
                            }
                        }
                    } else {
                        is_dirty = true;
                    }

                    if is_dirty {
                        *view.borrow_mut() = new_view;
                        node.view.as_ref().unwrap().clone()
                    } else {
                        return;
                    }
                } else {
                    node.view = Some(Rc::new(RefCell::new(new_view)));
                    node.view.as_ref().unwrap().clone()
                }
            };
            view.borrow_mut().any_view();

            self.build_cx.borrow().children.get(key).cloned()
        };

        if let Some(children) = children {
            for child_key in children {
                self.compose(child_key);
            }
        }

        TASK_CONTEXT.try_with(|cx| *cx.borrow_mut() = None).unwrap();
    }

    /// Build the initial composition.
    pub fn build(&mut self) {
        self.compose(self.root);
        GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow_mut().dirty.clear())
            .unwrap();
    }

    /// Rebuild the composition from it's previous state.
    pub async fn rebuild(&mut self) {
        TASK_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.task_cx.clone()))
            .unwrap();

        loop {
            let fut =
                async {
                    self.rx.next().await;
                };

            if futures::poll!(Box::pin(fut)).is_ready() {
                break;
            }

            self.task_cx.local_pool.borrow_mut().run_until_stalled();
        }

        self.compose(self.root);
        GLOBAL_CONTEXT
            .try_with(|cx| cx.borrow_mut().dirty.clear())
            .unwrap();
    }
}
