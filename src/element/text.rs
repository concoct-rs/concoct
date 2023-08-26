use super::Element;
use crate::ElementContext;

pub struct TextElement<'a> {
    content: &'a str,
}

impl<'a> TextElement<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }
}

impl Element for TextElement<'_> {
    fn build(&self, cx: &mut ElementContext) {
        let elem = cx.document.create_text_node(self.content);
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();
    }
}
