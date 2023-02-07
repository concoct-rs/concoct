use concoct::{
    composable::{
        column,
        container::{modifier::Gap, ContainerModifier},
        material::button::Button,
        row, state, Text,
    },
    modify::ModifyExt,
    render::run,
    DevicePixels, Modifier,
};
use taffy::{
    prelude::Size,
    style::{AlignItems, Dimension, JustifyContent},
};

fn app() {
    column(
        Modifier
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .flex_grow(1.)
            .gap(Gap::default().height(Dimension::Points(20.dp())))
            .size(Size::default()),
        || {
            let count = state(|| 0);

            Text::build(count.get().cloned().to_string()).font_size(80.dp());

            row(
                Modifier.gap(Gap::default().width(Dimension::Points(20.dp()))),
                move || {
                    Button::new(move || *count.get().as_mut() -= 1, || Text::new("Less"));

                    Button::new(move || *count.get().as_mut() += 1, || Text::new("More"));
                },
            )
        },
    )
}

#[tokio::main]
async fn main() {
    run(app)
}
