#![feature(type_alias_impl_trait)]

use concoct::{
    view::{Canvas, View},
    Renderer,
};
use skia_safe::{Color4f, Paint};
use taffy::prelude::Size;

fn app() -> impl View<(), ()> {
    let mut a = Canvas::new(|_layout, canvas| {
        let color = Color4f::new(1., 0., 0., 1.);

        canvas.draw_circle((50., 50.), 50., &Paint::new(color, None));
    });
    a.style.size = Size::from_points(200., 200.);

    let mut b = Canvas::new(|_layout, canvas| {
        let color = Color4f::new(0., 1., 0., 1.);

        canvas.draw_circle((100., 100.), 100., &Paint::new(color, None));
    });
    b.style.size = Size::from_points(200., 200.);
    (a, b)
}

fn main() {
    let view = app();
    Renderer {}.run(view);
}
