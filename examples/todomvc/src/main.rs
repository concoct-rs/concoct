use concoct::{
    view::{lazy, once, View},
    web::{attr, class, on, value, Element, EventExt, Html, Web},
    Modify,
};
use std::mem;

enum Event {
    None,
    UpdateInput(String),
    Add,
    Remove(u32),
    Check(u32),
    Edit { id: u32, is_editing: bool },
    Update { id: u32, content: String },
}

impl Event {
    fn edit(id: u32, is_editing: bool) -> Self {
        Self::Edit { id, is_editing }
    }
}

#[derive(Clone, Hash)]
struct Todo {
    id: u32,
    content: String,
    is_editing: bool,
    is_completed: bool,
}

#[derive(Default)]
struct Model {
    input: String,
    next_id: u32,
    unused_ids: Vec<u32>,
    todos: Vec<Todo>,
}

impl Model {
    pub fn get_mut(&mut self, id: u32) -> &mut Todo {
        self.todos.iter_mut().find(|todo| todo.id == id).unwrap()
    }
}

fn view(state: &Model) -> impl View<Web<Event>> {
    Html::div(
        class("todomvc-wrapper"),
        (
            Html::section(
                class("todoapp"),
                (
                    lazy(&state.input, view_input(state)),
                    lazy(&state.todos, view_entries(state)),
                ),
            ),
            once(view_footer()),
        ),
    )
}

fn view_input(state: &Model) -> impl View<Web<Event>> {
    Html::header(
        class("header"),
        (
            Html::h1((), "Todos"),
            Html::input(
                (
                    class("new-todo"),
                    attr("placeholder", "What needs to be done?"),
                    attr("autofocus", "True"),
                    attr("name", "newTodo"),
                    value(state.input.clone()),
                    on("input", |event| {
                        event.prevent_default();
                        Event::UpdateInput(event.event_target_value())
                    }),
                    on_enter(|| Event::Add),
                ),
                (),
            ),
        ),
    )
}

fn view_entries(state: &Model) -> impl View<Web<Event>> {
    Html::ul(
        class("todo-list"),
        state
            .todos
            .iter()
            .map(|todo| (todo.id, view_entry(&todo)))
            .collect::<Vec<_>>(),
    )
}

fn view_entry(todo: &Todo) -> impl View<Web<Event>> {
    let id = todo.id;
    let class_list = if todo.is_completed {
        if todo.is_editing {
            "completed editing"
        } else {
            "completed"
        }
    } else if todo.is_editing {
        "editing"
    } else {
        ""
    };

    Html::li(
        class(class_list),
        (
            Html::div(
                class("view"),
                (
                    Html::input(
                        (
                            class("toggle"),
                            attr("type", "checkbox"),
                            attr("checked", todo.is_completed.to_string()),
                            on("click", move |_| Event::Check(id)),
                        ),
                        (),
                    ),
                    Html::label(
                        on("click", move |_| Event::edit(id, true)),
                        todo.content.clone(),
                    ),
                    Html::button(
                        (class("destroy"), on("click", move |_| Event::Remove(id))),
                        (),
                    ),
                ),
            ),
            Html::input(
                (
                    class("edit"),
                    value(todo.content.clone()),
                    attr("name", "content"),
                    on("input", move |event| {
                        event.prevent_default();
                        Event::Update {
                            id,
                            content: event.event_target_value(),
                        }
                    }),
                    on("blur", move |_| Event::edit(id, false)),
                    on_enter(move || Event::edit(id, false)),
                ),
                (),
            ),
        ),
    )
}

fn view_footer() -> impl View<Web<Event>> {
    Html::footer(class("info"), Html::p((), "Click to edit a todo"))
}

fn on_enter(f: impl Fn() -> Event + 'static) -> impl Modify<Web<Event>, Element> {
    on("keydown", move |event| {
        if event.event_key_code() == 13 {
            f()
        } else {
            Event::None
        }
    })
}

fn main() {
    concoct::web::run(
        Model::default(),
        |state, event| match event {
            Event::None => {}
            Event::UpdateInput(value) => {
                state.input = value;
            }
            Event::Add => {
                let content = mem::take(&mut state.input);
                let id = state.unused_ids.pop().unwrap_or_else(|| {
                    let id = state.next_id;
                    state.next_id += 1;
                    id
                });
                state.todos.push(Todo {
                    id,
                    content,
                    is_editing: false,
                    is_completed: false,
                });
            }
            Event::Check(id) => {
                let todo = state.get_mut(id);
                todo.is_completed = !todo.is_completed;
            }
            Event::Edit { id, is_editing } => {
                let todo = state.get_mut(id);
                todo.is_editing = is_editing;
            }
            Event::Update { id, content } => {
                let todo = state.get_mut(id);
                todo.content = content;
            }
            Event::Remove(id) => {
                if let Some(idx) = state.todos.iter().position(|todo| todo.id == id) {
                    state.todos.remove(idx);
                    state.unused_ids.push(id);
                }
            }
        },
        view,
    )
}
