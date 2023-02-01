use concoct::{container, render::run, text, Modifier};

fn app() {
    container(Modifier::default(), || text("Hello World!"))
}

fn main() {
    run(app)
}
