use crate::{composable::interaction_source::InteractionSource, semantics::Handler, Event};
use accesskit::Node;
use winit::event::TouchPhase;

pub struct Scrollable<I, F> {
    interaction_source: I,
    on_delta: F,
    last_touch: Option<f64>,
}

impl<I, F> Scrollable<I, F> {
    pub fn new(interaction_source: I, on_click: F) -> Self {
        Self {
            interaction_source,
            on_delta: on_click,
            last_touch: None,
        }
    }
}

impl<I, F> Handler for Scrollable<I, F>
where
    I: InteractionSource<f64>,
    F: FnMut(f64) + 'static,
{
    fn handle(&mut self, _node: &Node, event: Event) {
        match event {
            Event::Touch(touch) => match touch.phase {
                TouchPhase::Started => {
                    self.last_touch = Some(touch.location.y);
                }
                TouchPhase::Moved => {
                    let last = self.last_touch.unwrap();
                    self.last_touch = Some(touch.location.y);

                    (self.on_delta)(touch.location.y - last)
                }
                _ => {}
            },
            Event::MouseWheel { delta } => (self.on_delta)(delta),
            _ => {}
        }
    }
}
