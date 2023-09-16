use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent};

pub trait EventExt {
    fn target_value(&self) -> String;

    fn key_code(&self) -> u32;
}

impl EventExt for Event {
    fn target_value(&self) -> String {
        self.target()
            .unwrap()
            .unchecked_into::<web_sys::HtmlInputElement>()
            .value()
    }

    fn key_code(&self) -> u32 {
        self.unchecked_ref::<KeyboardEvent>().key_code()
    }
}
