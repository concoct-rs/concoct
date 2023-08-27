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
    header().then((
        h1().then("Todos"),
        input().modify((
            value(state.title.clone()),
            on("input", |event| {
                event.prevent_default();
                Event::UpdateField(event_target_value(&event))
            }),
            on("keydown", |event| {
                if event_key_code(&event) == 13 {
                    Event::AddTodo
                } else {
                    Event::None
                }
            }),
        )),
        p().then(state.title.clone()),
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
