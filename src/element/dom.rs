use wasm_bindgen::{prelude::Closure, JsCast};
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
    type State = Closure<dyn FnMut()>;

    fn build(&self, cx: &mut ElementContext) -> Self::State {
        let elem = cx.document.create_element(self.tag).unwrap();

        let update = cx.update.clone();
        let f: Closure<dyn FnMut()> = Closure::new(move || {
            update.borrow_mut()();
        });
        elem.add_event_listener_with_callback("click", f.as_ref().unchecked_ref())
            .unwrap();

        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();

        cx.stack.push(elem);
        self.child.build(cx);
        cx.stack.pop();

        f
    }
}
