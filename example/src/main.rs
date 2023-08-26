use concoct::{App, Html, View};

fn counter() -> impl View {
    "Hello World!"
}

fn main() {
   let mut app = App::new();
   app.run(counter());
}
