use concoct::{
    native::{Canvas, Native},
    view::View,
};

enum Event {
    Increment,
    Decrement,
}

fn counter(_count: &i32) -> impl View<Native<Event>> {
    (Canvas {}, Canvas {})
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
