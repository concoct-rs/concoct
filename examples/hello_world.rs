use concoct::{composable::material::button, container, render::run, Modifier};

fn app() {
    container(
        Modifier::default().flex_direction(taffy::style::FlexDirection::Column),
        || {
            button("Hello", || {});
            button("World", || {});
        },
    )
}

fn main() {
    run(app)
}
