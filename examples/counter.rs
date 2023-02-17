use concoct::{
    composable::{material::button::Button, state, Container, Text},
    dimension::{DevicePixels, Size},
    render::run,
    View,
};
use taffy::style::{AlignItems, Dimension, JustifyContent};
use tracing::Level;

fn app() {
    Container::build_column(|| {
        let count = state(|| 0);

        Text::build(count.cloned().to_string())
            .font_size(80.dp())
            .view();

        Container::build_row(move || {
            Button::new(|| Text::new("Less"))
                .on_press(move || *count.get().as_mut() -= 1)
                .view();

            Button::new(|| Text::new("More"))
                .on_press(move || *count.get().as_mut() += 1)
                .view();
        })
        .gap(Size::default().width(Dimension::Points(20.dp())))
        .view()
    })
    .align_items(AlignItems::Center)
    .justify_content(JustifyContent::Center)
    .flex_grow(1.)
    .gap(Size::default().height(Dimension::Points(20.dp())))
    .view()
}

#[tokio::main]
async fn main() {
    let collector = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(collector).unwrap();

    run(app)
}
