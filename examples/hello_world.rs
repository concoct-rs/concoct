use concoct::{container, render::run, text, Modifier};

fn app() {
    container(
        Modifier::default().flex_direction(taffy::style::FlexDirection::Column),
        || {
            text(Modifier::default(), "A");

            container(
                Modifier::default().flex_direction(taffy::style::FlexDirection::Column),
                || {
                    text(Modifier::default(), "B");
                },
            );
        },
    )
}

fn main() {
    run(app)
}
