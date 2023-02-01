use concoct::{container, render::run, text, Modifier};

fn app() {
    container(Modifier::default(), || {
        text(Modifier::default(), "Hello");
        text(Modifier::default(), "World!");
    })
}

fn main() {
    run(app)
}
