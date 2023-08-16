use accesskit::Role;
use concoct::{
    view::{clickable, remember, Canvas, View},
    Renderer,
};
use skia_safe::{Color4f, Paint};
use taffy::prelude::Size;

fn circle(radius: f32) -> impl View<f32> {
    Canvas::new(move |_layout, canvas| {
        let color = Color4f::new(1., 0., 0., 1.);
        canvas.draw_circle((radius, radius), radius, &Paint::new(color, None));
    })
    .size(Size::from_points(radius * 2., radius * 2.))
}

fn app() -> impl View<()> {
    remember(
        || 50.,
        |radius: &mut f32| clickable(Role::Button, |r: &mut f32| *r *= 2., circle(*radius)),
    )
}

fn main() {
    Renderer::default().run(app);
}
