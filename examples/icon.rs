use concoct::{
    composable::{Icon, Text},
    render::run,
};
use skia_safe::{Color4f, Paint, RGB};

fn app() {
    Icon::build(
        include_str!("../icon.svg"),
        Paint::new(Color4f::from(RGB::from((0, 255, 0))), None),
    )
    .view();
}

#[tokio::main]
async fn main() {
    run(app)
}
