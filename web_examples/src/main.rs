use concoct::{use_state, web::html, IntoComposable};

fn app() -> impl IntoComposable {
    let mut count = use_state(|| 0);
    (
        "High five count: ",
        html("Up High").on_click(|| count += 1),
        html("Down low").on_click(|| count -= 1),
    )
}

fn main() {
    concoct::web::run(app)
}
