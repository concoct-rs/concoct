use crate::Semantics;
use accesskit::{NodeId, Role};
use skia_safe::{Canvas, Color4f, Paint};
use std::marker::PhantomData;
use taffy::{
    prelude::{Layout, Size},
    style::Dimension,
};

pub mod handler;
pub use handler::HandlerModifier;

pub trait Modify<T> {
    fn modify(&mut self, value: &mut T);

    fn semantics(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}

    fn paint(&mut self, _layout: &Layout, _canvas: &mut Canvas) {}

    fn remove(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}
}

pub struct Modifier;

impl<T> Modify<T> for Modifier {
    fn modify(&mut self, _value: &mut T) {}
}

pub struct Chain<T, A: Modify<T>, B: Modify<T>> {
    a: A,
    b: B,
    _marker: PhantomData<T>,
}

impl<T, A: Modify<T>, B: Modify<T>> Modify<T> for Chain<T, A, B> {
    fn modify(&mut self, value: &mut T) {
        self.a.modify(value);
        self.b.modify(value);
    }

    fn semantics(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        self.a.semantics(node_id, semantics);
        self.b.semantics(node_id, semantics);
    }

    fn paint(&mut self, layout: &Layout, canvas: &mut Canvas) {
        self.a.paint(layout, canvas);
        self.b.paint(layout, canvas);
    }

    fn remove(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        self.a.remove(node_id, semantics);
        self.b.remove(node_id, semantics);
    }
}

pub trait ModifyExt<T>: Modify<T> {
    fn background_color(self, color: impl Into<Color4f>) -> Chain<T, Self, BackgroundColor>
    where
        Self: Sized,
    {
        self.chain(BackgroundColor {
            color: color.into(),
        })
    }

    fn chain<B>(self, modify: B) -> Chain<T, Self, B>
    where
        Self: Sized,
        B: Modify<T>,
    {
        Chain {
            a: self,
            b: modify,
            _marker: PhantomData,
        }
    }

    fn draw<F>(self, f: F) -> Chain<T, Self, Draw<F>>
    where
        Self: Sized,
        F: FnMut(&Layout, &mut Canvas),
    {
        self.chain(Draw { f })
    }

    fn size(self, size: Size<Dimension>) -> Chain<T, Self, Size<Dimension>>
    where
        Self: Sized,
        T: AsMut<Size<Dimension>>,
    {
        self.chain(size)
    }
}

impl<T, M: Modify<T>> ModifyExt<T> for M {}

impl<T> Modify<T> for Role
where
    T: AsMut<Role>,
{
    fn modify(&mut self, value: &mut T) {
        *value.as_mut() = *self;
    }
}

pub struct BackgroundColor {
    color: Color4f,
}

impl<T> Modify<T> for BackgroundColor {
    fn modify(&mut self, _value: &mut T) {}

    fn paint(&mut self, layout: &Layout, canvas: &mut Canvas) {
        canvas.draw_rect(
            skia_safe::Rect::new(
                layout.location.x,
                layout.location.y,
                layout.location.x + layout.size.width,
                layout.location.y + layout.size.height,
            ),
            &Paint::new(self.color, None),
        );
    }
}

pub struct Draw<F> {
    f: F,
}

impl<T, F> Modify<T> for Draw<F>
where
    F: FnMut(&Layout, &mut Canvas),
{
    fn modify(&mut self, _value: &mut T) {}

    fn paint(&mut self, layout: &Layout, canvas: &mut Canvas) {
        (self.f)(layout, canvas)
    }
}
