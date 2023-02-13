use crate::Semantics;
use accesskit::NodeId;
use skia_safe::{Canvas, Color4f, Paint, Point, RRect, Rect};

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

pub struct Then<A: Modify, B: Modify> {
    a: A,
    b: B,
}

impl<A: Modify, B: Modify> Modify for Then<A, B> {
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
    fn background_color(self, color: impl Into<Color4f>) -> Then<Self, BackgroundColor>
    where
        Self: Sized,
    {
        self.then(BackgroundColor {
            color: color.into(),
        })
    }

    fn then<B>(self, modify: B) -> Then<Self, B>
    where
        Self: Sized,
        B: Modify,
    {
        Then { a: self, b: modify }
    }

    fn clip(self, radii: [Point; 4]) -> Then<Self, Clip>
    where
        Self: Sized,
    {
        self.then(Clip { radii })
    }

    fn draw<F>(self, f: F) -> Then<Self, Draw<F>>
    where
        Self: Sized,
        F: FnMut(&Layout, &mut Canvas),
    {
        self.then(Draw { f })
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

pub struct Clip {
    radii: [Point; 4],
}

impl Modify for Clip {
    fn paint(&mut self, layout: &Layout, canvas: &mut Canvas) {
        let rrect = RRect::new_rect_radii(
            Rect::new(
                layout.location.x,
                layout.location.y,
                layout.location.x + layout.size.width,
                layout.location.y + layout.size.height,
            ),
            &self.radii,
        );
        canvas.clip_rrect(rrect, None, true);
    }
}
