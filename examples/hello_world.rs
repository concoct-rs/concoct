use concoct::{composable::text, render::run, Modifier};
use skia_safe::Typeface;

fn app() {
    text(
        Modifier::default().typeface(Typeface::new("serif", Default::default()).unwrap()),
        "Hello World!",
    );
}

fn main() {
    run(app)
}
