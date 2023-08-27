use crate::{Attribute, Context};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Element, Event};

pub fn on<F, E>(name: &str, make: F) -> OnAttr<F>
where
    F: Fn(Event) -> E + 'static,
    E: 'static,
{
    OnAttr { name, make }
}

pub struct OnAttr<'a, F> {
    name: &'a str,
    make: F,
}

impl<'a, F, E> Attribute<E> for OnAttr<'a, F>
where
    F: Fn(Event) -> E + 'static,
    E: 'static,
{
    type State = (&'a str, Closure<dyn FnMut(Event)>);

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        let update = cx.update.clone();

        let f: Closure<dyn FnMut(Event)> = Closure::new(move |event| {
            update.borrow_mut().as_mut().unwrap()((self.make)(event));
        });
        elem.add_event_listener_with_callback(self.name, f.as_ref().unchecked_ref())
            .unwrap();
        (self.name, f)
    }

    fn rebuild(self, _cx: &mut Context<E>, _elem: &mut Element, _state: &mut Self::State) {}
}
