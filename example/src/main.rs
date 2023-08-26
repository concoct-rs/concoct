use concoct::{App, Html, View};

fn counter() -> impl View {
    Html::new("h2")
}

fn main() {
   let mut app = App::new();
   app.run(counter());
}
