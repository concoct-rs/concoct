use concoct::{
    native::{canvas, Native, text},
    view::View,
};
use skia_safe::{Color4f, Paint};

enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Native<Event>> {
    text(count.to_string())
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
