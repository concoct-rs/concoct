use concoct::view::html::{button, event_target_value, h1, header, input, on, p, value};
use concoct::view::View;
use wasm_bindgen::JsCast;

enum Event {
    UpdateField(String),
}

#[derive(Default)]
struct State {
    title: String,
}

fn view_input(state: &State) -> impl View<Event> {
    header((
        h1("Todos"),
        input(()).modify((
            value(state.title.clone()),
            on("input", |event| {
                event.prevent_default();
                let val = event_target_value(&event);
                Event::UpdateField(val)
            }),
        )),
        p(state.title.clone()),
    ))
}

fn app(state: &State) -> impl View<Event> {
    view_input(state)
}

fn main() {
    concoct::run(
        State::default(),
        |state, event| match event {
            Event::UpdateField(value) => {
                state.title = value;
            }
        },
        app,
    )
}
