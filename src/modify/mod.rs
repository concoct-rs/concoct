use crate::{Event, Semantics};
use accesskit::{NodeId, Role};

pub mod container;

mod modifier;
pub use modifier::Modifier;
use skia_safe::{Canvas, Color4f, Paint};
use taffy::{
    prelude::{Layout, Rect, Size},
    style::{AlignItems, Dimension, FlexDirection, Style},
};

pub mod keyboard_input;

pub trait Modify<T> {
    fn modify(&mut self, value: &mut T);

    fn semantics(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}

    fn paint(&mut self, _layout: &Layout, _canvas: &mut Canvas) {}
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

    fn paint(&mut self, layout: &Layout, canvas: &mut Canvas) {
        self.a.paint(layout, canvas);
        self.b.paint(layout, canvas);
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
    F: FnMut() + 'static,
{
    fn modify(&mut self, _value: &mut T) {}

    fn semantics(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        if let Some(mut f) = self.f.take() {
            semantics.handlers.insert(
                node_id,
                Box::new(move |event| match event {
                    Event::Action(_) => f(),
                    _ => {}
                }),
            );
        }
    }
}

impl<T: AsMut<Style>> Modify<T> for AlignItems {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().align_items = *self;
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

#[derive(Default)]
pub struct Padding {
    rect: Rect<Dimension>,
}

impl Padding {
    pub fn left(mut self, value: Dimension) -> Self {
        self.rect.left = value;
        self
    }

    pub fn right(mut self, value: Dimension) -> Self {
        self.rect.right = value;
        self
    }

    pub fn horizontal(self, value: Dimension) -> Self {
        self.left(value).right(value)
    }
}

impl<T: AsMut<Style>> Modify<T> for Padding {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().padding = self.rect;
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

pub struct FlexGrow {
    value: f32,
}

impl<T: AsMut<Style>> Modify<T> for FlexGrow {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().flex_grow = self.value;
    }
}
