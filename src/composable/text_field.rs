use super::Text;
use crate::{modify::HandlerModifier, Modifier, View};
use winit::event::{ElementState, VirtualKeyCode};

pub struct TextField<F> {
    value: String,
    on_change: F,
}

impl<F> TextField<F>
where
    F: FnMut(&str) + 'static,
{
    pub fn build(value: impl Into<String>, on_change: F) -> Self {
        Self {
            value: value.into(),
            on_change,
        }
    }

    #[track_caller]
    pub fn new(value: impl Into<String>, on_change: F) {
        Self::build(value, on_change).view()
    }

    fn push_char(&mut self, c: char) {
        self.value.push(c);
        (self.on_change)(&self.value);
    }
}

impl<F> View for TextField<F>
where
    F: FnMut(&str) + 'static,
{
    #[track_caller]
    fn view(mut self) {
        Text::build(self.value.clone())
            .modifier(Modifier.keyboard_handler(move |state, virtual_keycode| {
                if state == ElementState::Pressed {
                    match virtual_keycode {
                        VirtualKeyCode::A => self.push_char('a'),
                        VirtualKeyCode::Back => {
                            self.value.pop();
                            (self.on_change)(&self.value)
                        }
                        _ => {}
                    }
                }
            }))
            .view()
    }
}
