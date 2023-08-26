use concoct::view::html::{button, h1, on};
use concoct::view::View;

enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Event> {
    (
        h1([], count.to_string()),
        button([on("click", Event::Increment)], "More"),
        button([on("click", Event::Decrement)], "Less"),
    )
}

fn main() {
    concoct::run(
        0,
        |count, event| match event {
            Event::Increment => *count += 1,
            Event::Decrement => *count -= 1,
        },
        counter,
    );
}
