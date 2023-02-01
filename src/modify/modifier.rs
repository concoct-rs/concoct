use super::{container::MergeDescendants, Chain, Clickable};
use accesskit::{Action, Role};
use taffy::{prelude::Size, style::{Dimension, FlexDirection}};
use std::marker::PhantomData;

pub struct Modifier<T, M> {
    pub modify: M,
    _marker: PhantomData<T>,
}

impl<T> Default for Modifier<T, ()> {
    fn default() -> Self {
        Self::new(())
    }
}

impl<T, M> Modifier<T, M> {
    pub fn new(modify: M) -> Self {
        Self {
            modify,
            _marker: PhantomData,
        }
    }

    pub fn chain<B>(self, modify: B) -> Modifier<T, Chain<M, B>> {
        Modifier::new(Chain {
            a: self.modify,
            b: modify,
        })
    }

    pub fn clickable<F: FnMut(Action) + 'static>(
        self,
        on_click: F,
    ) -> Modifier<T, Chain<M, Clickable<F>>> {
        self.chain(Clickable { f: Some(on_click) })
    }

    pub fn flex_direction(self, flex_direction: FlexDirection) -> Modifier<T, Chain<M, FlexDirection>> {
        self.chain(flex_direction)
    }

    pub fn merge_descendants(self) -> Modifier<T, Chain<M, MergeDescendants>> {
        self.chain(MergeDescendants)
    }

    pub fn role(self, role: Role) -> Modifier<T, Chain<M, Role>> {
        self.chain(role)
    }

    pub fn size(self, size: Size<Dimension>) -> Modifier<T, Chain<M, Size<Dimension>>> {
        self.chain(size)
    }
}
