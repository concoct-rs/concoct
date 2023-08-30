use concoct::{
    native::{canvas, Native},
    view::View,
};
use skia_safe::{Color4f, Paint};

enum Event {
    Increment,
    Decrement,
}

fn counter(_count: &i32) -> impl View<Native<Event>> {
    (
        canvas(|layout, canvas| {
            canvas.draw_circle(
                (layout.location.x, layout.location.y),
                100.,
                &Paint::new(Color4f::new(1., 0., 0., 1.), None),
            );
        }),
        canvas(|layout, canvas| {
            canvas.draw_circle(
                (layout.location.x, layout.location.y),
                100.,
                &Paint::new(Color4f::new(0., 1., 0., 1.), None),
            );
        }),
    )
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
