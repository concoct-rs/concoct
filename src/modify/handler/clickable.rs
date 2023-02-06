use crate::{semantics::Handler, Event};
use accesskit::Node;
use winit::event::{ElementState, TouchPhase};

pub struct ClickHandler<F> {
    pub(crate) on_click: F,
}

impl<F> Handler for ClickHandler<F>
where
    F: FnMut() + 'static,
{
    fn handle(&mut self, node: &Node, event: Event) {
        match event {
            Event::Action(_) => (self.on_click)(),
            Event::MouseInput { state, cursor } => match state {
                ElementState::Pressed => {}
                ElementState::Released => {
                    let bounds = node.bounds.unwrap();
                    if cursor.x > bounds.x0
                        && cursor.x < bounds.x1
                        && cursor.y > bounds.y0
                        && cursor.y < bounds.y1
                    {
                        (self.on_click)();
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
                        (self.on_click)();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
