use concoct::view::{Html, View};
use concoct::App;

fn counter(count: &u32) -> impl View {
    Html::new("h1", count.to_string())
}

fn main() {
    let mut app = App::new();
    app.run(0, |count| *count += 1, counter);
}
