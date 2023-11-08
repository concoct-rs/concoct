use std::ops::Deref;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

pub struct InputEvent {
    pub raw: web_sys::InputEvent,
}

impl From<web_sys::Event> for InputEvent {
    fn from(value: web_sys::Event) -> Self {
        Self {
            raw: value.unchecked_into(),
        }
    }
}

impl InputEvent {
    pub fn target(&self) -> Option<HtmlInputElement> {
        self.raw.target().map(|target| target.unchecked_into())
    }
}

impl Deref for InputEvent {
    type Target = web_sys::InputEvent;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

pub struct MouseEvent {
    pub raw: web_sys::MouseEvent,
}

impl From<web_sys::Event> for MouseEvent {
    fn from(value: web_sys::Event) -> Self {
        Self {
            raw: value.unchecked_into(),
        }
    }
}
