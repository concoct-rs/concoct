use crate::Modify;

use super::{
    container::MergeDescendants,
    keyboard_input::{KeyboardHandler, KeyboardInput},
    BackgroundColor, Chain, Clickable, FlexBasis, FlexGrow, FlexShrink, Gap, Margin,
    Padding,
};
use accesskit::Role;
use skia_safe::{Color4f, Typeface};
use std::marker::PhantomData;
use taffy::{
    prelude::{Rect, Size},
    style::{AlignItems, Dimension, FlexDirection, JustifyContent},
};

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

    pub fn align_items(self, align_items: AlignItems) -> Modifier<T, Chain<M, AlignItems>> {
        self.chain(align_items)
    }

    pub fn background_color(
        self,
        color: impl Into<Color4f>,
    ) -> Modifier<T, Chain<M, BackgroundColor>> {
        self.chain(BackgroundColor {
            color: color.into(),
        })
    }

    pub fn chain<B>(self, modify: B) -> Modifier<T, Chain<M, B>> {
        Modifier::new(Chain {
            a: self.modify,
            b: modify,
        })
    }

    pub fn clickable<F>(self, on_click: F) -> Modifier<T, Chain<M, Clickable<F>>>
    where
        F: FnMut() + 'static,
    {
        self.chain(Clickable { f: Some(on_click) })
    }

    pub fn flex_basis(self, dimension: Dimension) -> Modifier<T, Chain<M, FlexBasis>> {
        self.chain(FlexBasis { dimension })
    }

    pub fn flex_direction(
        self,
        flex_direction: FlexDirection,
    ) -> Modifier<T, Chain<M, FlexDirection>> {
        self.chain(flex_direction)
    }

    pub fn flex_grow(self, value: f32) -> Modifier<T, Chain<M, FlexGrow>> {
        self.chain(FlexGrow { value })
    }

    pub fn flex_shrink(self, value: f32) -> Modifier<T, Chain<M, FlexShrink>> {
        self.chain(FlexShrink { value })
    }



    pub fn gap(self, gap: Gap) -> Modifier<T, Chain<M, Gap>> {
        self.chain(gap)
    }

    pub fn justify_content(
        self,
        justify_content: JustifyContent,
    ) -> Modifier<T, Chain<M, JustifyContent>> {
        self.chain(justify_content)
    }

    pub fn keyboard_handler<H>(self, handler: H) -> Modifier<T, Chain<M, KeyboardInput<H>>>
    where
        H: KeyboardHandler + 'static,
    {
        self.chain(KeyboardInput::new(handler))
    }

    pub fn margin(self, rect: Rect<Dimension>) -> Modifier<T, Chain<M, Margin>> {
        self.chain(Margin { rect })
    }

    pub fn merge_descendants(self) -> Modifier<T, Chain<M, MergeDescendants>> {
        self.chain(MergeDescendants)
    }

    pub fn padding(self, padding: Padding) -> Modifier<T, Chain<M, Padding>> {
        self.chain(padding)
    }

    pub fn role(self, role: Role) -> Modifier<T, Chain<M, Role>> {
        self.chain(role)
    }

    pub fn size(self, size: Size<Dimension>) -> Modifier<T, Chain<M, Size<Dimension>>> {
        self.chain(size)
    }

  
}

impl<T, M: Modify<T>> Modify<T> for Modifier<T, M> {
    fn modify(&mut self, value: &mut T) {
        self.modify.modify(value)
    }

    fn semantics(&mut self, node_id: accesskit::NodeId, semantics: &mut crate::Semantics) {
        self.modify.semantics(node_id, semantics)
    }

    fn paint(&mut self, layout: &taffy::prelude::Layout, canvas: &mut skia_safe::Canvas) {
        self.modify.paint(layout, canvas)
    }

    fn remove(&mut self, node_id: accesskit::NodeId, semantics: &mut crate::Semantics) {
        self.modify.remove(node_id, semantics)
    }
}
