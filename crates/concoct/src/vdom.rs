use crate::{Model, View};
use slotmap::{DefaultKey, SlotMap};
use std::{
    any::{self, Any},
    cell::{Cell, RefCell},
    fmt,
    marker::PhantomData,
    rc::Rc,
};

pub struct Context<'a> {
    pub(crate) tree: &'a RefCell<&'a mut Tree>,
    pub(crate) is_empty: Rc<Cell<bool>>,
    pub(crate) parent_key: Option<DefaultKey>,
}

impl Context<'_> {
    pub(crate) fn build<V, A>(&self, view: &mut V) -> DefaultKey
    where
        V: View<A>,
        A: 'static,
    {
        let model = view.build();

        let pod = Pod {
            model,
            view_name: any::type_name::<V>(),
            _marker: PhantomData,
        };
        let node = Node {
            model: Rc::new(RefCell::new(pod)),
            children: Vec::new(),
        };
        let key = self.tree.borrow_mut().nodes.insert(node);

        let is_empty = self.is_empty.clone();
        let model = self.tree.borrow().nodes[key].model.clone();
        let model_ref = model.borrow();
        let mut body = view.body(self, model_ref.as_any().downcast_ref().unwrap());

        if !is_empty.get() {
            if let Some(parent_key) = self.parent_key {
                self.tree.borrow_mut().nodes[parent_key].children.push(key);
            }

            let child_cx = Context {
                tree: self.tree,
                is_empty: Default::default(),
                parent_key: Some(key),
            };
            child_cx.build(&mut body);
        } else {
            self.tree.borrow_mut().nodes.remove(key);
        }

        key
    }
}

trait AnyModel {
    fn as_any(&self) -> &dyn Any;

    fn view_name(&self) -> &'static str;
}

struct Pod<M, A> {
    model: M,
    view_name: &'static str,
    _marker: PhantomData<A>,
}

impl<M, A> AnyModel for Pod<M, A>
where
    M: Model<A>,
    A: 'static,
{
    fn as_any(&self) -> &dyn Any {
        &self.model
    }

    fn view_name(&self) -> &'static str {
        self.view_name
    }
}

struct Node {
    model: Rc<RefCell<dyn AnyModel>>,
    children: Vec<DefaultKey>,
}

#[derive(Default)]
pub(crate) struct Tree {
    nodes: SlotMap<DefaultKey, Node>,
    root_key: Option<DefaultKey>,
}

pub struct VirtualDom<V> {
    root: V,
    tree: Tree,
}

impl<V> VirtualDom<V> {
    pub fn new(view: V) -> Self {
        Self {
            root: view,
            tree: Tree::default(),
        }
    }

    pub fn build(&mut self)
    where
        V: View,
    {
        let cx = Context {
            tree: &RefCell::new(&mut self.tree),
            is_empty: Default::default(),
            parent_key: None,
        };
        let key = cx.build(&mut self.root);
        self.tree.root_key = Some(key);
    }
}

struct NodeDebugger<'a> {
    node: &'a Node,
    tree: &'a Tree,
}

impl fmt::Debug for NodeDebugger<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple(self.node.model.borrow().view_name());
        for child_key in &self.node.children {
            let node = &self.tree.nodes[*child_key];
            let debugger = Self {
                node,
                tree: self.tree,
            };
            tuple.field(&debugger);
        }
        tuple.finish()
    }
}

impl<V> fmt::Debug for VirtualDom<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("VirtualDom");
        if let Some(root_key) = self.tree.root_key {
            let node = &self.tree.nodes[root_key];
            let debugger = NodeDebugger {
                node,
                tree: &self.tree,
            };
            tuple.field(&debugger);
        }
        tuple.finish()
    }
}
