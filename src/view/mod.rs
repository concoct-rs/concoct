use crate::ElementContext;
use web_sys::Text;

mod html;
pub use html::Html;

pub trait View {
    type State;

    fn build(self, cx: &mut ElementContext) -> Self::State;

    fn rebuild(self, cx: &mut ElementContext, state: &mut Self::State);
}

impl View for String {
    type State = (String, Text);

    fn build(self, cx: &mut ElementContext) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();
        (self, elem)
    }

    fn rebuild(self, _cx: &mut ElementContext, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
    }
}
