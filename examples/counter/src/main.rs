use concoct::{
    view::{once, View},
    web::{on, Html, Web},
};

enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Web<Event>> {
    (
        Html::h1((), count.to_string()),
        once(Html::button(on("click", |_| Event::Increment), "More")),
        once(Html::button(on("click", |_| Event::Decrement), "Less")),
    )
}

fn main() {
    concoct::web::run(
        0,
        |count, event| match event {
            Event::Increment => *count += 1,
            Event::Decrement => *count -= 1,
        },
        counter,
    );
}
