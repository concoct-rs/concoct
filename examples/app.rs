#![feature(type_alias_impl_trait)]

use concoct::{
    view::{Canvas, View},
    Renderer,
};
use skia_safe::{Color4f, Paint};
use taffy::prelude::Size;

fn app() -> impl View<(), ()> {
    let mut c = Canvas::new(|_layout, canvas| {
        let is_red = false;
        let color = if is_red {
            Color4f::new(1., 0., 0., 1.)
        } else {
            Color4f::new(0., 1., 0., 1.)
        };

        canvas.draw_circle((50., 50.), 50., &Paint::new(color, None));
    });
    c.style.size = Size::from_points(200., 200.);
    c
}

fn main() {
    let view = app();
    Renderer {}.run(view);
}
