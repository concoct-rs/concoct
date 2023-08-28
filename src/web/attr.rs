use super::Web;
use crate::{web::Context, Modify};
use web_sys::Element;

pub fn class<T>(value: T) -> Attr<&'static str, T> {
    attr("class", value)
}

pub fn attr<T, U>(name: T, value: U) -> Attr<T, U> {
    Attr { name, value }
}

pub struct Attr<T, U> {
    name: T,
    value: U,
}

impl<E, T: AsRef<str>, U: AsRef<str>> Modify<Web<E>, Element> for Attr<T, U> {
    type State = ();

    fn build(self, _cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        elem.set_attribute(self.name.as_ref(), self.value.as_ref())
            .unwrap()
    }

    fn rebuild(self, _cx: &mut Context<E>, elem: &mut Element, _state: &mut Self::State) {
        elem.set_attribute(self.name.as_ref(), self.value.as_ref())
            .unwrap()
    }
}
