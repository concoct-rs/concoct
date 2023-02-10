use concoct::{
    composable::{key, Container, Text},
    render::run,
};

fn app() {
    Container::build_column(|| {
        for i in 0..10 {
            key(i, || Container::row(|| Text::new("Hello World!")));
        }
    })
    .view();
}

#[tokio::main]
async fn main() {
    run(app)
}
