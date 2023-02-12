use concoct::{
    composable::{key, Container, Text},
    modify::HandlerModifier,
    render::run,
    Modifier, View,
};

fn app() {
    Container::build_column(|| {
        for i in 0..10 {
            key(i, || Container::row(|| Text::new("Hello World!")));
        }
    })
    .modifier(Modifier.scrollable(|delta| {
        dbg!(delta);
    }))
    .view();
}

#[tokio::main]
async fn main() {
    run(app)
}
