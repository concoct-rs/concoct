use crate::{
    AnyComposable, BuildContext, Composable, Inner, LocalContext, Node, TaskContext, TASK_CONTEXT,
};
use futures::pending;
use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use std::{any::Any, cell::RefCell, rc::Rc};
use tokio::sync::mpsc;

pub struct Composition {
    nodes: SlotMap<DefaultKey, Node>,
    children: SparseSecondaryMap<DefaultKey, Vec<DefaultKey>>,
    root: DefaultKey,
    task_cx: TaskContext,
    rx: mpsc::UnboundedReceiver<Box<dyn Any>>,
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

        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            nodes: composables,
            children: SparseSecondaryMap::new(),
            root,
            task_cx: TaskContext {
                local_set: Default::default(),
                tx,
            },
            rx,
        }
    }

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

        TASK_CONTEXT.try_with(|cx| *cx.borrow_mut() = None).unwrap();
    }

    pub async fn rebuild(&mut self) {
        TASK_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self.task_cx.clone()))
            .unwrap();

        let local_set = TASK_CONTEXT
            .try_with(|local_set| local_set.clone())
            .unwrap();

        loop {
            let fut = async {
                let mut g = local_set.borrow_mut();
                let fut = &mut *g.as_mut().unwrap().local_set.borrow_mut();
                fut.await;
            };
            if futures::poll!(Box::pin(fut)).is_pending() {
                pending!()
            } else {
                break;
            }
        }

        self.rx.recv().await;

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

        TASK_CONTEXT.try_with(|cx| *cx.borrow_mut() = None).unwrap();
    }
}
