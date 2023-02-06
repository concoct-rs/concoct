use concoct::{composable::{text, material::radio_button::{radio_button, RadioButtonModifier}}, render::run, Modifier};

fn app() {
    radio_button(Modifier.on_click(|| {
        dbg!("click!");
    }))
}

#[tokio::main]
async fn main() {
    run(app)
}
