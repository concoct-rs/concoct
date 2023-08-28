use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent};

pub trait EventExt {
    fn event_target_value(&self) -> String;

    fn event_key_code(&self) -> u32;
}

impl EventExt for Event {
    fn event_target_value(&self) -> String {
        self.target()
            .unwrap()
            .unchecked_into::<web_sys::HtmlInputElement>()
            .value()
    }

    fn event_key_code(&self) -> u32 {
        self.unchecked_ref::<KeyboardEvent>().key_code()
    }
}
