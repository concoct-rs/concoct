use concoct::{composable, Composable, Composer, Model};

#[derive(Default)]
struct App {
    name: String,
}

impl Model<()> for App {
    fn handle(&mut self, _msg: ()) {}
}

fn app(model: &App) -> impl Composable<()> {
    dbg!(&model.name);

    composable::lazy(
        &model.name,
        composable::from_fn(move |_| {
            dbg!("Changed");
        }),
    )
}

#[tokio::main]
async fn main() {
    let mut composer = Composer::new(App::default(), app);

    composer.compose();
    composer.recompose();
}
