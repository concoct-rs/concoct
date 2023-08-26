use concoct::view::{Html, View};
use concoct::App;

fn counter() -> impl View {
    Html::new("h1", "Hello World!")
}

fn main() {
    let mut app = App::new();
    app.run(counter());
}
