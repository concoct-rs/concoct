use crate::{semantics::Handler, Event};
use accesskit::Node;
use winit::event::{ElementState, VirtualKeyCode};

pub trait KeyboardHandler {
    fn handle_keyboard_input(&mut self, state: ElementState, virtual_keycode: VirtualKeyCode);
}

impl<F: FnMut(ElementState, VirtualKeyCode)> KeyboardHandler for F {
    fn handle_keyboard_input(&mut self, state: ElementState, virtual_keycode: VirtualKeyCode) {
        self(state, virtual_keycode)
    }
}

pub struct KeyboardInputHandler<H> {
    handler: H,
}

impl<H> KeyboardInputHandler<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }
}

impl<H: KeyboardHandler> Handler for KeyboardInputHandler<H> {
    fn handle(&mut self, _node: &Node, event: Event) {
        if let Event::KeyboardInput { state, key_code } = event {
            self.handler.handle_keyboard_input(state, key_code)
        }
    }
}
