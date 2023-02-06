use concoct::{
    composable::{
        column,
        container::{modifier::Gap, ContainerModifier},
        material::button,
        row, state,
        text::{text, TextModifier},
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
                    button(
                        Modifier,
                        || text(Modifier, "More"),
                        move || *count.get().as_mut() += 1,
                    );

                    button(
                        Modifier,
                        || text(Modifier, "Less"),
                        move || *count.get().as_mut() -= 1,
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
