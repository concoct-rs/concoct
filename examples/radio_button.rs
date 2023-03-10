use concoct::{composable::material::RadioButton, render::run, View};

fn app() {
    RadioButton::build()
        .on_click(|| {
            dbg!("click!");
        })
        .view()
}

#[tokio::main]
async fn main() {
    run(app)
}
