use crate::{
    composable::interaction_source::InteractionSource, semantics::Handler, Composable, Event,
};
use accesskit::{kurbo::Rect, Node};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, TouchPhase},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickInteration {
    Press,
    Release,
    Cancel,
}

pub struct ClickHandler<I, F> {
    pub interaction_source: I,
    pub on_click: F,
    pub is_pressed: bool,
}

impl<I, F> ClickHandler<I, F> {
    pub fn new(interaction_source: I, on_click: F) -> Self {
        Self {
            interaction_source,
            on_click,
            is_pressed: false,
        }
    }
}
impl<I, F> ClickHandler<I, F>
where
    I: InteractionSource<ClickInteration>,
    F: Composable + 'static,
{
    pub fn press(&mut self, pos: PhysicalPosition<f64>, bounds: Rect) {
        if is_intersection(pos, bounds) {
            self.is_pressed = true;
            self.interaction_source.emit(ClickInteration::Press);
        }
    }

    pub fn release(&mut self, pos: PhysicalPosition<f64>, bounds: Rect) {
        if self.is_pressed {
            if is_intersection(pos, bounds) {
                (self.on_click).compose();
                self.interaction_source.emit(ClickInteration::Release);
            } else {
                self.interaction_source.emit(ClickInteration::Cancel);
            }

            self.is_pressed = false;
        }
    }
}

impl<I, F> Handler for ClickHandler<I, F>
where
    I: InteractionSource<ClickInteration>,
    F: Composable + 'static,
{
    fn handle(&mut self, node: &Node, event: Event) {
        match event {
            Event::Action(_) => (self.on_click).compose(),
            Event::MouseInput { state, cursor } => {
                let bounds = node.bounds.unwrap();
                match state {
                    ElementState::Pressed => self.press(cursor, bounds),
                    ElementState::Released => self.release(cursor, bounds),
                }
            }
            Event::Touch(touch) => {
                let bounds = node.bounds.unwrap();
                match touch.phase {
                    TouchPhase::Started => self.press(touch.location, bounds),
                    TouchPhase::Moved | TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.release(touch.location, bounds)
                    }
                }
            }
            _ => {}
        }
    }
}

fn is_intersection(pos: PhysicalPosition<f64>, bounds: Rect) -> bool {
    pos.x > bounds.x0 && pos.x < bounds.x1 && pos.y > bounds.y0 && pos.y < bounds.y1
}
