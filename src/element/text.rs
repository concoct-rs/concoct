use super::Element;
use crate::ElementContext;

pub struct TextElement {
    content: String,
}

impl TextElement {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl Element for TextElement {
    type State = ();

    fn build(&self, cx: &mut ElementContext) {
        let elem = cx.document.create_text_node(&self.content);
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();
    }
}
