use concoct::{web::html, IntoComposable};

fn app() -> impl IntoComposable {
    html().attr("class", "main")
}

fn main() {
    concoct::web::run(app)
}
