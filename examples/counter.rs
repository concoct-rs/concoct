use concoct::{
    composable::{column, material::button, row, state, text},
    modify::Gap,
    render::run,
    DevicePixels, Modifier,
};
use taffy::style::{AlignItems, Dimension, JustifyContent};

fn app() {
    column(
        Modifier::default()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .flex_grow(1.)
            .gap(Gap::default().height(Dimension::Points(20.dp()))),
        || {
            let count = state(|| 0);

            text(Modifier::default().font_size(80.dp()), count.get().cloned().to_string());

            row(
                Modifier::default().gap(Gap::default().width(Dimension::Points(20.dp()))),
                move || {
                    button(Modifier::default(), "More", move || {
                        *count.get().as_mut() += 1
                    });

                    button(Modifier::default(), "Less", move || {
                        *count.get().as_mut() -= 1
                    });
                },
            )
        },
    )
}

fn main() {
    run(app)
}
