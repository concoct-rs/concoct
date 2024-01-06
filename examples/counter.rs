use concoct::{composable, Composable, Composer, Model};
use std::time::Duration;
use tokio::time;

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

    composable::once(composable::from_fn(|cx| {
        let sender = cx.clone();

        sender.send(Message::Decrement);
        tokio::spawn(async move {
            loop {
                time::sleep(Duration::from_secs(1)).await;
                sender.send(Message::Increment)
            }
        });
    }))
}

#[tokio::main]
async fn main() {
    let mut composer = Composer::new(App::default(), app);

    composer.compose();
    loop {
        composer.handle().await;
        composer.recompose();
    }
}
