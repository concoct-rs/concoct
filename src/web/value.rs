use super::Web;
use crate::Modify;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};

/// Set the value attribute of an element.
pub fn value<T>(value: T) -> Value<T> {
    Value { value }
}

/// View for the [`value`] function.
pub struct Value<T> {
    value: T,
}

impl<E, T: AsRef<str>> Modify<Web<E>, Element> for Value<T> {
    type State = ();

    fn build(self, _cx: &mut Web<E>, elem: &mut Element) -> Self::State {
        elem.unchecked_ref::<HtmlInputElement>()
            .set_value(self.value.as_ref());
    }

    fn rebuild(self, _cx: &mut Web<E>, elem: &mut Element, _state: &mut Self::State) {
        elem.unchecked_ref::<HtmlInputElement>()
            .set_value(self.value.as_ref());
    }
}
