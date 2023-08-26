use super::View;
use crate::Context;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::Element;

pub fn on<E>(name: &'static str, event: E) -> Attribute<E> {
    Attribute::On { name, event }
}

pub enum Attribute<E> {
    On { name: &'static str, event: E },
}

impl<E: 'static> Attribute<E> {
    pub fn add(
        self,
        cx: &mut Context<E>,
        elem: &mut Element,
    ) -> (&'static str, Closure<dyn FnMut()>) {
        match self {
            Self::On { name, event } => {
                let update = cx.update.clone();
                let mut msg_cell = Some(event);
                let f: Closure<dyn FnMut()> = Closure::new(move || {
                    update.borrow_mut().as_mut().unwrap()(msg_cell.take().unwrap());
                });
                elem.add_event_listener_with_callback(name, f.as_ref().unchecked_ref())
                    .unwrap();
                (name, f)
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

impl<'a, A, V, E> View<E> for Html<'a, A, V>
where
    A: IntoIterator<Item = Attribute<E>>,
    V: View<E>,
    E: 'static,
{
    type State = (Vec<(&'static str, Closure<dyn FnMut()>)>, Element, V::State);

    fn build(self, cx: &mut Context<E>) -> Self::State {
        let mut elem = cx.document.create_element(self.tag).unwrap();
        cx.insert(&elem);

        let fs = self
            .attributes
            .into_iter()
            .map(|attr| attr.add(cx, &mut elem))
            .collect();

        cx.stack.push((elem, 0));
        let state = self.child.build(cx);
        let (elem, _) = cx.stack.pop().unwrap();

        (fs, elem, state)
    }

    fn rebuild(self, cx: &mut Context<E>, state: &mut Self::State) {
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

        cx.skip();
        cx.stack.push((state.1.clone(), 0));
        self.child.rebuild(cx, &mut state.2);
        cx.stack.pop();
    }

    fn remove(_cx: &mut Context<E>, state: &mut Self::State) {
        state.1.remove();
    }
}
