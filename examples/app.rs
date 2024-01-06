use concoct::{composable, Composable, Composer, Model};

#[derive(Debug)]
enum Message {
    Increment,
    Decrement,
}

#[derive(Default)]
struct App {
    count: i32,
}

impl Model<Message> for App {
    fn handle(&mut self, msg: Message) {
        match msg {
            Message::Decrement => self.count -= 1,
            Message::Increment => self.count += 1,
        }
    }
}

fn app(model: &App) -> impl Composable<Message> {
    dbg!(model.count);

    let count = model.count;
    composable::lazy(
        &model.count,
        composable::from_fn(move |_| {
            dbg!(count);
        }),
    )
}

#[tokio::main]
async fn main() {
    let mut composer = Composer::new(App::default(), app);

    composer.compose();
    composer.recompose();
}
