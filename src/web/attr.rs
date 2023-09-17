use std::borrow::Cow;

use super::Web;
use crate::Modify;
use web_sys::Element;

/// Set the class attribute for an element.
pub fn class(value: impl Into<Cow<'static, str>>) -> Attr {
    attr("class", value)
}

/// Set a stringly-typed attribute for an element.
pub fn attr(name: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) -> Attr {
    Attr {
        name: name.into(),
        value: value.into(),
    }
}

/// View for the [`attr`] function.
pub struct Attr {
    name: Cow<'static, str>,
    value: Cow<'static, str>,
}

impl<E> Modify<Web<E>, Element> for Attr {
    type State = ();

    fn build(self, _cx: &mut Web<E>, elem: &mut Element) -> Self::State {
        elem.set_attribute(self.name.as_ref(), self.value.as_ref())
            .unwrap()
    }

    fn rebuild(self, _cx: &mut Web<E>, elem: &mut Element, _state: &mut Self::State) {
        elem.set_attribute(self.name.as_ref(), self.value.as_ref())
            .unwrap()
    }
}
