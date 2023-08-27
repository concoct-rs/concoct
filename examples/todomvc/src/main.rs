use concoct::attr::{event_key_code, event_target_value, on, value};
use concoct::view::html::{h1, header, input, li, p, span, ul};
use concoct::view::View;
use std::mem;

enum Event {
    None,
    UpdateField(String),
    AddTodo,
    RemoveTodo(u32),
}

#[derive(Default)]
struct State {
    title: String,
    next_id: u32,
    todos: Vec<(u32, String)>,
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
                .map(|(key, todo)| {
                    let key = *key;
                    (
                        key,
                        li().then((
                            todo.clone(),
                            span()
                                .modify(on("click", move |_| Event::RemoveTodo(key)))
                                .then("X"),
                        )),
                    )
                })
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
                state.todos.push((state.next_id, title));
                state.next_id += 1;
            }
            Event::RemoveTodo(key) => {
                if let Some(idx) = state.todos.iter().position(|(k, _)| key == *k) {
                    state.todos.remove(idx);
                }
            }
        },
        app,
    )
}
