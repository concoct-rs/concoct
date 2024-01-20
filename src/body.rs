use crate::{Node, Scope, Tree, View};
use std::{cell::RefCell, rc::Rc};

pub trait Body: 'static {
    fn into_tree(self) -> impl Tree;
}

impl<B: Body> Body for Option<B> {
    fn into_tree(self) -> impl Tree {
        self.map(|me| me.into_tree())
    }
}

pub struct Child<B> {
    cell: Rc<RefCell<Option<B>>>,
}

impl<B> Child<B> {
    pub fn new(body: B) -> Self {
        Self {
            cell: Rc::new(RefCell::new(Some(body))),
        }
    }
}

impl<B> Clone for Child<B> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<B: Body> Body for Child<B> {
    fn into_tree(self) -> impl Tree {
        self.cell.take().unwrap().into_tree()
    }
}

pub struct Empty;

impl Body for Empty {
    fn into_tree(self) -> impl Tree {
        self
    }
}

impl<V: View> Body for V {
    fn into_tree(self) -> impl Tree {
        Node {
            view: self,
            body: None,
            builder: |me: &'static V| me.body().into_tree(),
            scope: Scope::default(),
            key: None,
        }
    }
}

macro_rules! impl_body_for_tuple {
    ($($t:tt : $idx:tt),*) => {
        impl<$($t: Body),*> Body for ($($t),*) {
            fn into_tree(self) -> impl Tree {
                ($(  self.$idx.into_tree() ),*)

            }
        }
    };
}

impl_body_for_tuple!(V1: 0, V2: 1);
impl_body_for_tuple!(V1: 0, V2: 1, V3: 2);
impl_body_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3);
impl_body_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4);
impl_body_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5);
impl_body_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6);
impl_body_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7);
impl_body_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8);
impl_body_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8, V10: 9);
