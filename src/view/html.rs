use super::View;
use crate::ElementContext;
use wasm_bindgen::{prelude::Closure, JsCast};

pub struct Html<'a, V> {
    tag: &'a str,
    child: V,
}

impl<'a, V> Html<'a, V> {
    pub fn new(tag: &'a str, child: V) -> Self {
        Self { tag, child }
    }
}

impl<'a, V> View for Html<'a, V>
where
    V: View,
{
    type State = (Closure<dyn FnMut()>, V::State);

    fn build(self, cx: &mut ElementContext) -> Self::State {
        let elem = cx.document.create_element(self.tag).unwrap();

        let update = cx.update.clone();
        let f: Closure<dyn FnMut()> = Closure::new(move || {
            update.borrow_mut().as_mut().unwrap()();
        });
        elem.add_event_listener_with_callback("click", f.as_ref().unchecked_ref())
            .unwrap();

        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();

        cx.stack.push(elem);
        let state = self.child.build(cx);
        cx.stack.pop();

        (f, state)
    }

    fn rebuild(self, cx: &mut ElementContext, state: &mut Self::State) {
        self.child.rebuild(cx, &mut state.1)
    }
}
