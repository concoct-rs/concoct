use concoct::{
    view::{Canvas, View},
    EventHandler, Renderer,
};
use skia_safe::{Color4f, Paint};
use taffy::prelude::Size;

fn circle(radius: f32) -> impl View<f32, ()> {
    Canvas::new(move |_layout, canvas| {
        let color = Color4f::new(1., 0., 0., 1.);
        canvas.draw_circle((radius, radius), radius, &Paint::new(color, None));
    })
    .size(Size::from_points(radius * 2., radius * 2.))
}

fn app(r: &mut f32) -> impl View<f32, ()> {
    EventHandler::new(|r: &mut f32| *r *= 2., circle(*r))
}

fn main() {
    Renderer::default().run(app, 50.);
}
