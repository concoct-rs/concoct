use concoct::{container, render::run, text, Modifier, composable::material::button};

fn app() {
    container(Modifier::default().flex_direction(taffy::style::FlexDirection::Column), || {
        button("Hello", || {});
        button("World", || {});
    })
}

fn main() {
    run(app)
}
