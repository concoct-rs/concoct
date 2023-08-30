use concoct::{
    native::Native,
    view::{once, View},
};

enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Native<Event>> {
    ()
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
