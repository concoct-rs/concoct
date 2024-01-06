<div align="center">
  <h1>Concoct</h1>
  
 <a href="https://crates.io/crates/concoct">
    <img src="https://img.shields.io/crates/v/concoct?style=flat-square"
    alt="Crates.io version" />
  </a>
  <a href="https://docs.rs/concoct">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
   <a href="https://github.com/concoct-rs/concoct/actions">
    <img src="https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg"
      alt="CI status" />
  </a>
</div>

<div align="center">
 <a href="https://github.com/concoct-rs/concoct/tree/main/examples">Examples</a>
</div>

<br />

Concoct is a framework for user-interfaces in Rust.

This crate provides a diffing-engine and state management system for any backend.

```rust
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
```

## Installation
The easiest way to get started is using the `full` feature flag.

```
cargo add concoct --features full
```

To see a list of the available features flags that can be enabled, check our [docs](https://docs.rs/concoct/latest/concoct/#feature-flags).
