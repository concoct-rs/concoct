use concoct::view::html::{button, h1, header, input, on};
use concoct::view::View;

enum Event {}

fn view_input() -> impl View<Event> {
    header((h1("Todos"), input(())))
}

fn app(state: &()) -> impl View<Event> {
    view_input()
}

fn main() {
    concoct::run((), |_, _| {}, app)
}
