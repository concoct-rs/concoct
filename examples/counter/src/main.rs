use concoct::{
    view::{self, View},
    web::{on, Html, Web},
};

enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Web<Event>> {
    (
        Html::h1().view(count.to_string()),
        view::once(
            Html::button()
                .on("click", |_| Event::Increment)
                .view("More"),
        ),
        view::once(
            Html::button()
                .on("click", |_| Event::Decrement)
                .view("Less"),
        ),
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
