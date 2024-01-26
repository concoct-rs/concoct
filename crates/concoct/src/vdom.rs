use std::{
    cell::{Cell, RefCell},
    ops::DerefMut,
    rc::Rc,
};

use slotmap::{DefaultKey, SlotMap};

use crate::{build_inner, rebuild_inner, Channel, Node, Scope, View};

/// Virtual DOM for a view.
pub struct VirtualDom<T, V> {
    content: V,
    nodes: Rc<RefCell<SlotMap<DefaultKey, Node>>>,
    channel: Rc<RefCell<Channel<T>>>,
    root_key: Option<DefaultKey>,
}

impl<T, V> VirtualDom<T, V> {
    /// Create a new virtual dom from its content.
    pub fn new(content: V) -> Self {
        Self {
            content,
            nodes: Rc::default(),
            channel: Rc::new(RefCell::new(Channel {
                updates: Vec::new(),
                waker: None,
            })),
            root_key: None,
        }
    }

    /// Build the initial content.
    pub fn build(&mut self)
    where
        T: 'static,
        V: View<T> + DerefMut<Target = T>,
    {
        let node = Node::default();
        let root_key = self.nodes.borrow_mut().insert(node.clone());
        self.root_key = Some(root_key);

        let channel = self.channel.clone();
        let cx = Scope {
            key: root_key,
            node,
            update: Rc::new(move |f| {
                let mut channel_ref = channel.borrow_mut();
                channel_ref.updates.push(f);
                if let Some(waker) = channel_ref.waker.take() {
                    waker.wake();
                }
            }),
            is_empty: Cell::new(false),
            nodes: self.nodes.clone(),
            contexts: Default::default(),
        };
        build_inner(&mut self.content, &cx)
    }

    /// Rebuild the content from the last build
    ///
    /// ## Panics
    /// This function will panic if no initial build has been performed.
    pub async fn rebuild(&mut self)
    where
        T: 'static,
        V: View<T> + DerefMut<Target = T>,
    {
        futures::future::poll_fn(|cx| {
            self.channel.borrow_mut().waker = Some(cx.waker().clone());

            let mut is_updated = false;
            loop {
                let mut channel_ref = self.channel.borrow_mut();
                if let Some(update) = channel_ref.updates.pop() {
                    update(&mut self.content);
                    is_updated = true;
                } else {
                    break;
                }
            }

            if is_updated {
                let root_key = self.root_key.unwrap();
                let node = self.nodes.borrow()[root_key].clone();

                let channel = self.channel.clone();
                let cx = Scope {
                    key: root_key,
                    node,
                    update: Rc::new(move |f| {
                        let mut channel_ref = channel.borrow_mut();
                        channel_ref.updates.push(f);
                        if let Some(waker) = channel_ref.waker.take() {
                            waker.wake();
                        }
                    }),
                    is_empty: Cell::new(false),
                    nodes: self.nodes.clone(),
                    contexts: Default::default(),
                };
                rebuild_inner(&mut self.content, &cx);
            }

            std::task::Poll::Pending
        })
        .await
    }
}
