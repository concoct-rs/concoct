use crate::{Attribute, Context};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};

pub fn class<T>(value: T) -> StringAttr<&'static str, T> {
    attr("class", value)
}

pub fn attr<T, U>(name: T, value: U) -> StringAttr<T, U> {
    StringAttr { name, value }
}

pub struct StringAttr<T, U> {
    name: T,
    value: U,
}

impl<E, T: AsRef<str>, U: AsRef<str>> Attribute<E> for StringAttr<T, U> {
    type State = ();

    fn build(self, _cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        elem.set_attribute(self.name.as_ref(), self.value.as_ref()).unwrap()
    }

    fn rebuild(self, _cx: &mut Context<E>, elem: &mut Element, _state: &mut Self::State) {
        elem.set_attribute(self.name.as_ref(), self.value.as_ref()).unwrap()
    }
}
