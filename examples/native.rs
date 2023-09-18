use concoct::{
    native::{view::text, Native},
    view::View,
};

enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Native<Event>> {
    (text(count.to_string()), text("More"), text("Less"))
}

fn main() {
    concoct::native::run(
        0,
        |count, event| match event {
            Event::Increment => *count += 1,
            Event::Decrement => *count -= 1,
        },
        counter,
    );
}
