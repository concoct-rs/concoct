use std::time::Duration;

use concoct::{composable, Composable, Context};
use tokio::time;

#[derive(Debug)]
enum Message {
    Increment,
    Decrement,
}

fn app() -> impl Composable<Message> {
    composable::once(composable::from_fn(|cx| {
        let sender = cx.clone();

        sender.send(Message::Increment);
        tokio::spawn(async move {
            time::sleep(Duration::from_secs(1)).await;
            sender.send(Message::Decrement)
        });
    }))
}

#[tokio::main]
async fn main() {
    let (mut cx, mut rx) = Context::new();
    let mut state = app().compose(&mut cx);
    app().recompose(&mut cx, &mut state);

    dbg!(rx.recv().await);
    dbg!(rx.recv().await);
}
