use concoct::{
    attr::on,
    view::{Html, View},
};

enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Event> {
    (
        Html::h1((), count.to_string()),
        Html::button(on("click", |_| Event::Increment), "More"),
        Html::button(on("click", |_| Event::Decrement), "Less"),
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
