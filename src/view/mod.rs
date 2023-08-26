use crate::{
    element::{Element, TextElement},
    BuildContext, Id,
};

pub use html::Html;
mod html;

pub trait View {
    type State;

    type Element: Element;

    fn build(&self, cx: &mut BuildContext) -> (Id, Self::State, Self::Element);
}

impl View for String {
    type State = ();

    type Element = TextElement;

    fn build(&self, cx: &mut BuildContext) -> (Id, Self::State, Self::Element) {
        let id = cx.insert();
        let elem = TextElement::new(self.clone());
        (id, (), elem)
    }
}
