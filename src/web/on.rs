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
    F: FnMut(Event) -> E + 'static,
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
    F: FnMut(Event) -> E + 'static,
    E: 'static,
{
    type State = (Cow<'static, str>, Closure<dyn FnMut(Event)>);

    fn build(mut self, cx: &mut Web<E>, elem: &mut Element) -> Self::State {
        let update_cell = cx.update.clone();
        let closure: Closure<dyn FnMut(Event)> = Closure::new(move |event| {
            let msg = (self.handler)(event);

            let mut update = update_cell.borrow_mut();
            let update_fn = update.as_mut().unwrap();
            update_fn(msg);
        });

        elem.add_event_listener_with_callback(&self.name, closure.as_ref().unchecked_ref())
            .unwrap();

        (self.name, closure)
    }

    fn rebuild(mut self, cx: &mut Web<E>, elem: &mut Element, state: &mut Self::State) {
        let update_cell = cx.update.clone();
        let closure: Closure<dyn FnMut(Event)> = Closure::new(move |event| {
            let msg = (self.handler)(event);

            let mut update = update_cell.borrow_mut();
            let update_fn = update.as_mut().unwrap();
            update_fn(msg);
        });

        elem.remove_event_listener_with_callback(&state.0, state.1.as_ref().unchecked_ref())
            .unwrap();
        elem.add_event_listener_with_callback(&self.name, closure.as_ref().unchecked_ref())
            .unwrap();

        *state = (self.name, closure);
    }
}
