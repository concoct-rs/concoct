use concoct::view::{Attribute, Html, View};
use concoct::App;

enum Message {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Message> {
    Html::new(
        "h1",
        [Attribute::On {
            event: "click",
            msg: Message::Decrement,
        }],
        count.to_string(),
    )
}

fn main() {
    let mut app = App::new();
    app.run(
        0,
        |count, msg| match msg {
            Message::Increment => *count += 1,
            Message::Decrement => *count -= 1,
        },
        counter,
    );
}
