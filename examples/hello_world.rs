use concoct::{composable::text, render::run, Modifier};

fn app() {
    text(Modifier, "Hello World!")
}

fn main() {
    run(app)
}
