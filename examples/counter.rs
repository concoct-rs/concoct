use concoct::{
    composable::{
        column,
        container::{modifier::Gap, ContainerModifier},
        material::{
            button::{self, Button},
            text,
        },
        row, state,
        text::TextModifier,
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

            text(
                Modifier.font_size(80.dp()),
                count.get().cloned().to_string(),
            );

            row(
                Modifier.gap(Gap::default().width(Dimension::Points(20.dp()))),
                move || {
                    Button::new(
                        move || *count.get().as_mut() -= 1,
                        || text(Modifier, "Less"),
                    );

                    Button::new(
                        move || *count.get().as_mut() += 1,
                        || text(Modifier, "More"),
                    );
                },
            )
        },
    )
}

#[tokio::main]
async fn main() {
    run(app)
}
