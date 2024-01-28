use slotmap::{DefaultKey, SlotMap};
use std::{any, fmt, marker::PhantomData};

pub trait Model<A = ()>: 'static {
    type Message: 'static;

    fn update(&mut self, msg: Self::Message) -> Option<A>;
}

impl<A> Model<A> for () {
    type Message = ();

    fn update(&mut self, msg: Self::Message) -> Option<A> {
        None
    }
}

pub trait View<A = ()> {
    type Message: 'static;

    type Model: Model<A, Message = Self::Message>;

    fn build(&mut self) -> Self::Model;

    fn body(&self, model: &Self::Model) -> impl View<Self::Message>;
}

impl<A> View<A> for () {
    type Message = ();

    type Model = ();

    fn build(&mut self) -> Self::Model {}

    fn body(&self, model: &Self::Model) -> impl View<Self::Message> {}
}

trait AnyModel {
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
{
    fn view_name(&self) -> &'static str {
        self.view_name
    }
}

struct Node {
    model: Box<dyn AnyModel>,
    children: Vec<DefaultKey>,
}

pub struct VirtualDom<V> {
    root: V,
    nodes: SlotMap<DefaultKey, Node>,
    root_key: Option<DefaultKey>,
}

impl<V> VirtualDom<V> {
    pub fn new(view: V) -> Self {
        Self {
            root: view,
            nodes: SlotMap::new(),
            root_key: None,
        }
    }

    pub fn build(&mut self)
    where
        V: View,
    {
        let model = self.root.build();
        let pod = Pod {
            model,
            view_name: any::type_name::<V>(),
            _marker: PhantomData,
        };
        let node = Node {
            model: Box::new(pod),
            children: Vec::new(),
        };
        let key = self.nodes.insert(node);
        self.root_key = Some(key);
    }
}

impl<V> fmt::Debug for VirtualDom<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("VirtualDom");
        if let Some(root_key) = self.root_key {
            let node = &self.nodes[root_key];
            tuple.field(&node.model.view_name());
        }
        tuple.finish()
    }
}
