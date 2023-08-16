use concoct::{
    view::{Canvas, View},
    Renderer,
};
use skia_safe::{Color4f, Paint};
use taffy::prelude::Size;

fn circle(radius: f32) -> impl View<(), ()> {
    Canvas::new(move |_layout, canvas| {
        let color = Color4f::new(1., 0., 0., 1.);
        canvas.draw_circle((radius, radius), radius, &Paint::new(color, None));
    })
    .size(Size::from_points(radius * 2., radius * 2.))
}

fn app() -> impl View<(), ()> {
    (circle(50.), circle(100.))
}

fn main() {
    let view = app();
    Renderer {}.run(view);
}
