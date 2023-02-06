use crate::{Event, Semantics};
use accesskit::{NodeId, Role};
use skia_safe::{Canvas, Color4f, Paint};
use std::marker::PhantomData;
use taffy::{
    prelude::{Layout, Size},
    style::Dimension,
};
use winit::event::{ElementState, TouchPhase};

pub mod keyboard_input;

pub mod text;
pub use text::TextModifier;

mod modifier;
pub use modifier::Modifier;

use self::keyboard_input::{KeyboardHandler, KeyboardInput};

pub trait Modify<T> {
    fn modify(&mut self, value: &mut T);

    fn semantics(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}

    fn paint(&mut self, _layout: &Layout, _canvas: &mut Canvas) {}

    fn remove(&mut self, _node_id: NodeId, _semantics: &mut Semantics) {}
}

impl<T> Modify<T> for () {
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

    fn clickable<F>(self, on_click: F) -> Chain<T, Self, Clickable<F>>
    where
        Self: Sized,
        F: FnMut() + 'static,
    {
        self.chain(Clickable { f: Some(on_click) })
    }

    fn keyboard_handler<H>(self, handler: H) -> Chain<T, Self, KeyboardInput<H>>
    where
        Self: Sized,
        H: KeyboardHandler + 'static,
    {
        self.chain(KeyboardInput::new(handler))
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
