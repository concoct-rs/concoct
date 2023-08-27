use concoct::attr::{attr, class, event_key_code, event_target_value, on, value};
use concoct::view::html::{
    button, div, footer, h1, header, input, label, li, p, section, span, ul,
};
use concoct::view::{lazy, View};
use std::mem;

enum Event {
    None,
    UpdateInput(String),
    Add,
    Remove(u32),
}

#[derive(Default)]
struct State {
    input: String,
    next_id: u32,
    unused_ids: Vec<u32>,
    todos: Vec<(u32, String)>,
}

fn view_input(state: &State) -> impl View<Event> {
    header().modify(attr("class", "header")).then((
        h1().then("Todos"),
        input().modify((
            attr("class", "new-todo"),
            attr("placeholder", "What needs to be done?"),
            attr("autofocus", "True"),
            attr("name", "newTodo"),
            value(state.input.clone()),
            on("input", |event| {
                event.prevent_default();
                Event::UpdateInput(event_target_value(&event))
            }),
            on("keydown", |event| {
                if event_key_code(&event) == 13 {
                    Event::Add
                } else {
                    Event::None
                }
            }),
        )),
    ))
}

fn view_footer() -> impl View<Event> {
    footer()
        .modify(class("info"))
        .then(p().then("Double-click to edit a todo"))
}

fn app(state: &State) -> impl View<Event> {
    (div().modify(attr("class", "todomvc-wrapper")).then((
        section().modify(attr("class", "todoapp")).then((
            lazy(state.input.clone(), view_input(state)),
            lazy(state.todos.clone(), view_entries(state)),
        )),
        lazy((), view_footer()),
    )),)
}

fn view_entries(state: &State) -> impl View<Event> {
    ul().modify(class("todo-list")).then(
        state
            .todos
            .iter()
            .map(|(id, content)| (*id, view_entry(*id, content.clone())))
            .collect::<Vec<_>>(),
    )
}

fn view_entry(id: u32, content: String) -> impl View<Event> {
    li().then(div().modify(class("view")).then((
        label().then(content),
        button().modify((class("destroy"), on("click", move |_| Event::Remove(id)))),
    )))
}

fn main() {
    concoct::run(
        State::default(),
        |state, event| match event {
            Event::None => {}
            Event::UpdateInput(value) => {
                state.input = value;
            }
            Event::Add => {
                let title = mem::take(&mut state.input);
                let id = state.unused_ids.pop().unwrap_or_else(|| {
                    let id = state.next_id;
                    state.next_id += 1;
                    id
                });
                state.todos.push((id, title));
            }
            Event::Remove(id) => {
                if let Some(idx) = state.todos.iter().position(|(k, _)| id == *k) {
                    state.todos.remove(idx);
                    state.unused_ids.push(id);
                }
            }
        },
        app,
    )
}
