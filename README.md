<p align="center">
  <img alt="logo" src="./logo.png">
</p>

<div align="center">
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

## Rust cross-platform reactive UI framework.

```rust
enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Web<Event>> {
    (
        Html::h1().view(count.to_string()),
        view::once(
            Html::button()
                .on("click", |_| Event::Increment)
                .view("More"),
        ),
        view::once(
            Html::button()
                .on("click", |_| Event::Decrement)
                .view("Less"),
        ),
    )
}

fn main() {
    concoct::web::run(
        0,
        |count, event| match event {
            Event::Increment => *count += 1,
            Event::Decrement => *count -= 1,
        },
        counter,
    );
}
```

## Features
 - Cross-platform components
 - Compile-time UI tree
 - Efficient view updates
 - Inspired by the elm and xilem architectures

## Getting started
### Web
Install [`trunk`](https://trunkrs.dev) or `wasm-pack` (this tutorial will show serving with trunk).

```
cargo add concoct --features web
```

Create an index.html file in the crate root
```html
<html>
    <body></body>
</html>
```

Create a main view and run it with Concoct
```rust
fn app(_state: &()) -> impl View<Web<()>> {
    Html::h1((), "Hello World!"),
}

fn main() {
    concoct::web::run(
        0,
        |_state, _event| {},
        app,
    );
}
```

```
trunk serve
```` 
All done! Check it out at `http://localhost:8080`
