use crate::{Event, Modify, Semantics};
use accesskit::NodeId;
use winit::event::{ElementState, VirtualKeyCode};

pub trait KeyboardHandler {
    fn handle_keyboard_input(&mut self, state: ElementState, virtual_keycode: VirtualKeyCode);
}

impl<F: FnMut(ElementState, VirtualKeyCode)> KeyboardHandler for F {
    fn handle_keyboard_input(&mut self, state: ElementState, virtual_keycode: VirtualKeyCode) {
        self(state, virtual_keycode)
    }
}

pub struct KeyboardInput<H> {
    handler: Option<H>,
}

impl<H> KeyboardInput<H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler: Some(handler),
        }
    }
}

impl<T, H> Modify<T> for KeyboardInput<H>
where
    H: KeyboardHandler + 'static,
{
    fn modify(&mut self, _value: &mut T) {}

    fn semantics(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        if let Some(mut handler) = self.handler.take() {
            semantics.handlers.insert(
                node_id,
                Box::new(move |event| {
                    if let Event::KeyboardInput { state, key_code } = event {
                        handler.handle_keyboard_input(state, key_code)
                    }
                }),
            );
        }
    }
}
