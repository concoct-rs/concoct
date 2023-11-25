use concoct::{html::Html, IntoComposable};

fn app() -> impl IntoComposable {
    Html {}
}

fn main() {
    concoct::html::run(app)
}
