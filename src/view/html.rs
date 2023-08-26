use super::View;
use crate::{element::DomElement, BuildContext, Id};

pub struct Html<'a, V> {
    tag: &'a str,
    child: V,
}

impl<'a, V> Html<'a, V> {
    pub fn new(tag: &'a str, child: V) -> Self {
        Self { tag, child }
    }
}

impl<'a, V: View> View for Html<'a, V> {
    type State = V::State;

    type Element = DomElement<'a, V::Element>;

    fn build(&self, cx: &mut BuildContext) -> (Id, Self::State, Self::Element) {
        let id = cx.insert();

        let (child_id, child_state, child_elem) = self.child.build(cx);

        let elem = DomElement::new(self.tag, child_elem, child_id);

        (id, child_state, elem)
    }
}
