use crate::Semantics;
use accesskit::{NodeId, Role};
use skia_safe::{Canvas, Color4f, Paint};
use std::marker::PhantomData;
use taffy::prelude::Layout;

pub mod handler;
pub use handler::HandlerModifier;

pub trait Modify {
    fn semantics(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}

    fn paint(&mut self, _layout: &Layout, _canvas: &mut Canvas) {}

    fn remove(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}
}

impl<M: Modify> Modify for Option<M> {
    fn semantics(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        if let Some(modify) = self {
            modify.semantics(node_id, semantics)
        }
    }

    fn paint(&mut self, layout: &Layout, canvas: &mut Canvas) {
        if let Some(modify) = self {
            modify.paint(layout, canvas)
        }
    }

    fn remove(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        if let Some(modify) = self {
            modify.remove(node_id, semantics)
        }
    }
}

pub struct Modifier;

impl Modify for Modifier {}

pub struct Chain<A: Modify, B: Modify> {
    a: A,
    b: B,
}

impl<A: Modify, B: Modify> Modify for Chain<A, B> {
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

pub trait ModifyExt: Modify {
    fn background_color(self, color: impl Into<Color4f>) -> Chain<Self, BackgroundColor>
    where
        Self: Sized,
    {
        self.chain(BackgroundColor {
            color: color.into(),
        })
    }

    fn chain<B>(self, modify: B) -> Chain<Self, B>
    where
        Self: Sized,
        B: Modify,
    {
        Chain { a: self, b: modify }
    }

    fn draw<F>(self, f: F) -> Chain<Self, Draw<F>>
    where
        Self: Sized,
        F: FnMut(&Layout, &mut Canvas),
    {
        self.chain(Draw { f })
    }
}

impl<M: Modify> ModifyExt for M {}

pub struct BackgroundColor {
    color: Color4f,
}

impl Modify for BackgroundColor {
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

impl<F> Modify for Draw<F>
where
    F: FnMut(&Layout, &mut Canvas),
{
    fn paint(&mut self, layout: &Layout, canvas: &mut Canvas) {
        (self.f)(layout, canvas)
    }
}
