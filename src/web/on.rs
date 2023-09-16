use std::borrow::Cow;

use super::Web;
use crate::Modify;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Element, Event};

/// Add an event listener to an element.
///
/// The event type `name` will listen for events using the `handler` function and
/// emit the returned messages.
pub fn on<F, E>(name: impl Into<Cow<'static, str>>, handler: F) -> On<F>
where
    F: Fn(Event) -> E + 'static,
    E: 'static,
{
    On {
        name: name.into(),
        handler,
    }
}

/// Modifier for the [`on`] function.
pub struct On<F> {
    /// Event name
    name: Cow<'static, str>,

    /// Function to handle an event and return a message.
    handler: F,
}

impl<F, E> Modify<Web<E>, Element> for On<F>
where
    F: Fn(Event) -> E + 'static,
    E: 'static,
{
    type State = (Cow<'static, str>, Closure<dyn FnMut(Event)>);

    fn build(self, cx: &mut Web<E>, elem: &mut Element) -> Self::State {
        let update_cell = cx.update.clone();
        let closure: Closure<dyn FnMut(Event)> = Closure::new(move |event| {
            let mut update = update_cell.borrow_mut();
            let update_fn = update.as_mut().unwrap();
            let msg = (self.handler)(event);
            update_fn(msg);
        });

        elem.add_event_listener_with_callback(&self.name, closure.as_ref().unchecked_ref())
            .unwrap();

        (self.name, closure)
    }

    fn rebuild(self, _cx: &mut Web<E>, _elem: &mut Element, _state: &mut Self::State) {}
}
