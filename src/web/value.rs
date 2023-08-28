use crate::{web::Context, Modify};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};
use super::Web;

pub fn value(value: String) -> Value {
    Value { value }
}

pub struct Value {
    value: String,
}

impl<E> Modify<Web<E>> for Value {
    type State = ();

    fn build(self, _cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        elem.unchecked_ref::<HtmlInputElement>()
            .set_value(&self.value);
    }

    fn rebuild(self, _cx: &mut Context<E>, elem: &mut Element, _state: &mut Self::State) {
        elem.unchecked_ref::<HtmlInputElement>()
            .set_value(&self.value);
    }
}
