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
fn app() -> impl View {
    let mut count = use_signal(|| 0);

    Html::div().view((
        move || format!("High five count: {}", count),
        Html::button().on_click(move || count += 1).view("Up high!"),
        Html::button()
            .on_click(move || count -= 1)
            .view("Down low!"),
    ))
}
```

## Features
 - Cross-platform components
 - Compile-time UI tree
 - Efficient view updates
 - Inspired by the elm and xilem architectures


### Components

```rust
fn button(label: impl View + 'static, on_click: impl FnMut() + 'static) -> impl View {
    Html::button().on_click(on_click).view(label)
}

fn app() -> impl View {
    let selection = use_signal(|| "A");

    Html::div().view((
        move || button("A", move || *selection.write() = "A"),
        move || button("B", move || *selection.write() = "B"),
        move || button("C", move || *selection.write() = "C"),
    ))
}
```
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
