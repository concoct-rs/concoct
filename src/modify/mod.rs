use crate::{composable::text::TextModifier, Event, Semantics};
use accesskit::{NodeId, Role};
use skia_safe::{Canvas, Color4f, Paint, Typeface};
use taffy::{
    prelude::{Layout, Rect, Size},
    style::{AlignItems, Dimension, FlexDirection, JustifyContent, Style},
};
use winit::event::{ElementState, TouchPhase};

pub mod container;

pub mod keyboard_input;

mod modifier;
pub use modifier::Modifier;

pub trait Modify<T> {
    fn modify(&mut self, value: &mut T);

    fn semantics(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}

    fn paint(&mut self, _layout: &Layout, _canvas: &mut Canvas) {}

    fn remove(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}
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

    fn remove(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        self.a.remove(node_id, semantics);
        self.b.remove(node_id, semantics);
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
                Box::new(move |node, event| match event {
                    Event::Action(_) => f(),
                    Event::MouseInput { state, cursor } => match state {
                        ElementState::Pressed => {}
                        ElementState::Released => {
                            let bounds = node.bounds.unwrap();
                            if cursor.x > bounds.x0
                                && cursor.x < bounds.x1
                                && cursor.y > bounds.y0
                                && cursor.y < bounds.y1
                            {
                                f();
                            }
                        }
                    },
                    Event::Touch(touch) => match touch.phase {
                        TouchPhase::Ended => {
                            let bounds = node.bounds.unwrap();
                            if touch.location.x > bounds.x0
                                && touch.location.x < bounds.x1
                                && touch.location.y > bounds.y0
                                && touch.location.y < bounds.y1
                            {
                                f();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }),
            );
        }
    }

    fn remove(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        semantics.handlers.remove(&node_id);
    }
}

impl<T: AsMut<Style>> Modify<T> for AlignItems {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().align_items = *self;
    }
}

impl<T: AsMut<Style>> Modify<T> for JustifyContent {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().justify_content = *self;
    }
}

impl<T: AsMut<Style>> Modify<T> for FlexDirection {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().flex_direction = *self;
    }
}

impl<T: AsMut<Style>> Modify<T> for Size<Dimension> {
    fn modify(&mut self, value: &mut T) {
        let size = &mut value.as_mut().size;

        if self.width != Dimension::Undefined {
            size.width = self.width;
        }

        if self.height != Dimension::Undefined {
            size.height = self.height;
        }
    }
}

#[derive(Default)]
pub struct Gap {
    size: Size<Dimension>,
}

impl Gap {
    pub fn width(mut self, value: Dimension) -> Self {
        self.size.width = value;
        self
    }

    pub fn height(mut self, value: Dimension) -> Self {
        self.size.height = value;
        self
    }
}

impl<T: AsMut<Style>> Modify<T> for Gap {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().gap = self.size;
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

    pub fn top(mut self, value: Dimension) -> Self {
        self.rect.top = value;
        self
    }

    pub fn bottom(mut self, value: Dimension) -> Self {
        self.rect.bottom = value;
        self
    }

    pub fn vertical(self, value: Dimension) -> Self {
        self.top(value).bottom(value)
    }
}

impl From<Dimension> for Padding {
    fn from(value: Dimension) -> Self {
        Self::default().horizontal(value).vertical(value)
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

pub struct FlexShrink {
    value: f32,
}

impl<T: AsMut<Style>> Modify<T> for FlexShrink {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().flex_shrink = self.value;
    }
}

pub struct FlexBasis {
    dimension: Dimension,
}

impl<T: AsMut<Style>> Modify<T> for FlexBasis {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().flex_basis = self.dimension;
    }
}

pub struct Margin {
    rect: Rect<Dimension>,
}

impl<T: AsMut<Style>> Modify<T> for Margin {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().margin = self.rect;
    }
}

impl Modify<TextModifier> for Typeface {
    fn modify(&mut self, value: &mut TextModifier) {
        value.typeface = self.clone();
    }
}

pub struct FontSize {
    value: f32,
}

impl Modify<TextModifier> for FontSize {
    fn modify(&mut self, value: &mut TextModifier) {
        value.font_size = self.value;
    }
}
