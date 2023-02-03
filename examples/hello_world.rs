use concoct::{composable::text, render::run, Modifier};

fn app() {
    text(Modifier::default(), "Hello World!");
}

fn main() {
    run(app)
}
