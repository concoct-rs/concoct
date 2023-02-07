use concoct::{composable::Text, render::run};

fn app() {
    Text::new("Hello World!")
}

#[tokio::main]
async fn main() {
    run(app)
}
