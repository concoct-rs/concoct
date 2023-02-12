use concoct::{
    composable::{container::Gap, material::button::Button, state, Container, Text},
    render::run,
    DevicePixels, View,
};
use taffy::{
    prelude::Size,
    style::{AlignItems, Dimension, JustifyContent},
};

fn app() {
    Container::build_column(|| {
        let count = state(|| 0);

        Text::build(count.get().cloned().to_string())
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
        .gap(Gap::default().width(Dimension::Points(20.dp())))
        .view()
    })
    .align_items(AlignItems::Center)
    .justify_content(JustifyContent::Center)
    .flex_grow(1.)
    .gap(Gap::default().height(Dimension::Points(20.dp())))
    .size(Size::default())
    .view()
}

#[tokio::main]
async fn main() {
    run(app)
}
