use crate::Semantics;
use accesskit::{Action, NodeId, Role};

pub mod container;

mod modifier;
pub use modifier::Modifier;
use taffy::{
    prelude::Size,
    style::{Dimension, Style, FlexDirection},
};

pub trait Modify<T> {
    fn modify(&mut self, value: &mut T);

    fn semantics(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}
}

impl<T> Modify<T> for () {
    fn modify(&mut self, _value: &mut T) {}
}

pub struct Chain<A, B> {
    a: A,
    b: B,
}

impl<T, A: Modify<T>, B: Modify<T>> Modify<T> for Chain<A, B> {
    fn modify(&mut self, value: &mut T) {
        self.a.modify(value);
        self.b.modify(value);
    }

    fn semantics(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        self.a.semantics(node_id, semantics);
        self.b.semantics(node_id, semantics);
    }
}

impl<T> Modify<T> for Role
where
    T: AsMut<Role>,
{
    fn modify(&mut self, value: &mut T) {
        *value.as_mut() = *self;
    }
}

pub struct Clickable<F> {
    f: Option<F>,
}

impl<T, F> Modify<T> for Clickable<F>
where
    F: FnMut(Action) + 'static,
{
    fn modify(&mut self, _value: &mut T) {}

    fn semantics(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        if let Some(f) = self.f.take() {
            semantics.handlers.insert(node_id, Box::new(f));
        }
    }
}

impl<T: AsMut<Style>> Modify<T> for FlexDirection {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().flex_direction = *self;
    }
}

impl<T: AsMut<Style>> Modify<T> for Size<Dimension> {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().size = *self;
    }
}
