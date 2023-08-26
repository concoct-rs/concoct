use std::iter;

use super::View;
use crate::Context;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::Element;

pub fn on<M>(event: &'static str, msg: M) -> Attribute<M> {
    Attribute::On { event, msg }
}

pub enum Attribute<M> {
    On { event: &'static str, msg: M },
}

impl<M: 'static> Attribute<M> {
    pub fn add(
        self,
        cx: &mut Context<M>,
        elem: &mut Element,
    ) -> (&'static str, Closure<dyn FnMut()>) {
        match self {
            Self::On { event, msg } => {
                let update = cx.update.clone();
                let mut msg_cell = Some(msg);
                let f: Closure<dyn FnMut()> = Closure::new(move || {
                    update.borrow_mut().as_mut().unwrap()(msg_cell.take().unwrap());
                });
                elem.add_event_listener_with_callback(event, f.as_ref().unchecked_ref())
                    .unwrap();
                (event, f)
            }
        }
    }
}

pub fn h1<A, V>(attributes: A, child: V) -> Html<'static, A, V> {
    Html::new("h1", attributes, child)
}

pub fn button<A, V>(attributes: A, child: V) -> Html<'static, A, V> {
    Html::new("button", attributes, child)
}

pub struct Html<'a, A, V> {
    tag: &'a str,
    attributes: A,
    child: V,
}

impl<'a, A, V> Html<'a, A, V> {
    pub fn new(tag: &'a str, attributes: A, child: V) -> Self {
        Self {
            tag,
            attributes,
            child,
        }
    }

    pub fn attributes<A2>(self, attributes: A2) -> Html<'a, A2, V> {
        Html::new(self.tag, attributes, self.child)
    }

    pub fn child<V2>(self, child: V2) -> Html<'a, A, V2> {
        Html::new(self.tag, self.attributes, child)
    }
}

impl<'a, A, V, M> View<M> for Html<'a, A, V>
where
    A: IntoIterator<Item = Attribute<M>>,
    V: View<M>,
    M: 'static,
{
    type State = (Vec<(&'static str, Closure<dyn FnMut()>)>, Element, V::State);

    fn build(self, cx: &mut Context<M>) -> Self::State {
        let mut elem = cx.document.create_element(self.tag).unwrap();
        let fs = self
            .attributes
            .into_iter()
            .map(|attr| attr.add(cx, &mut elem))
            .collect();

        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();

        cx.stack.push(elem);
        let state = self.child.build(cx);
        let elem = cx.stack.pop().unwrap();

        (fs, elem, state)
    }

    fn rebuild(self, cx: &mut Context<M>, state: &mut Self::State) {
        let fs = self
            .attributes
            .into_iter()
            .zip(state.0.iter())
            .map(|(attr, prev)| {
                state
                    .1
                    .remove_event_listener_with_callback(prev.0, prev.1.as_ref().unchecked_ref())
                    .unwrap();
                attr.add(cx, &mut state.1)
            })
            .collect();

        state.0 = fs;

        self.child.rebuild(cx, &mut state.2)
    }
}
