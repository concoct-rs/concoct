use concoct::view::html::{button, h1, on};
use concoct::view::View;

enum Message {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Message> {
    (
       
        button([on("click", Message::Increment)], "More"),
        if *count % 2 == 0 {
            Some(h1([], count.to_string()))
        } else {
            None
        },
        button([on("click", Message::Decrement)], "Less"),
    )
}

fn main() {
    concoct::run(
        0,
        |count, msg| match msg {
            Message::Increment => *count += 1,
            Message::Decrement => *count -= 1,
        },
        counter,
    );
}
