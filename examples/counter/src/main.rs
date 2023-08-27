use concoct::{
    attr::on,
    view::{
        html::{button, h1},
        View,
    },
};

enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Event> {
    (
        h1().then(count.to_string()),
        button()
            .modify(on("click", |_| Event::Increment))
            .then("More"),
        button()
            .modify(on("click", |_| Event::Decrement))
            .then("Less"),
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
