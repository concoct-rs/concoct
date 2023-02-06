use concoct::{composable::text, render::run, Modifier};

fn app() {
    text(Modifier, "Hello World!")
}

#[tokio::main]
async fn main() {
    run(app)
}
