use concoct::attr::{event_key_code, event_target_value, on, value};
use concoct::view::html::{h1, header, input, li, p, ul};
use concoct::view::View;
use slotmap::{DefaultKey, SlotMap};
use std::mem;

enum Event {
    None,
    UpdateField(String),
    AddTodo,
}

#[derive(Default)]
struct State {
    title: String,
    todos: SlotMap<DefaultKey, String>,
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
    (
        view_input(state),
        ul().then(
            state
                .todos
                .iter()
                .map(|(key, todo)| (key, li().then(todo.clone())))
                .collect::<Vec<_>>(),
        ),
    )
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
                let title = mem::take(&mut state.title);
                state.todos.insert(title);
            }
        },
        app,
    )
}
