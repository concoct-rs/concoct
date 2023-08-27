use concoct::view::html::{event_key_code, event_target_value, h1, header, input, on, p, value};
use concoct::view::View;
use std::mem;

enum Event {
    None,
    UpdateField(String),
    AddTodo,
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
            on("keydown", |event| {
                if event_key_code(&event) == 13 {
                    Event::AddTodo
                } else {
                    Event::None
                }
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
            Event::None => {}
            Event::UpdateField(value) => {
                state.title = value;
            }
            Event::AddTodo => {
                state.title = String::new();
            }
        },
        app,
    )
}
