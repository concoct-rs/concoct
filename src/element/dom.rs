use super::Element;
use crate::{ElementContext, Id};

pub struct DomElement<'a, C> {
    tag: &'a str,
    child: C,
    child_id: Id,
}

impl<'a, C> DomElement<'a, C> {
    pub fn new(tag: &'a str, child: C, child_id: Id) -> Self {
        Self {
            tag,
            child,
            child_id,
        }
    }
}

impl<'a, C> Element for DomElement<'a, C>
where
    C: Element,
{
    fn build(&self, cx: &mut ElementContext) {
        let elem = cx.document.create_element(self.tag).unwrap();
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();

        cx.stack.push(elem);
        self.child.build(cx);
        cx.stack.pop();
    }
}
