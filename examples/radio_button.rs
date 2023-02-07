use concoct::{
    composable::{
        material::radio_button::{radio_button, RadioButtonModifier},
        text,
    },
    render::run,
    Modifier,
};

fn app() {
    radio_button(Modifier.on_click(|| {
        dbg!("click!");
    }))
}

#[tokio::main]
async fn main() {
    run(app)
}
