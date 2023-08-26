use crate::Context;
use web_sys::Text;

mod html;
pub use html::{Attribute,Html};

pub trait View<M> {
    type State;

    fn build(self, cx: &mut Context<M>) -> Self::State;

    fn rebuild(self, cx: &mut Context<M>, state: &mut Self::State);
}

impl<M> View<M> for String {
    type State = (String, Text);

    fn build(self, cx: &mut Context<M>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();
        (self, elem)
    }

    fn rebuild(self, _cx: &mut Context<M>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
    }
}
