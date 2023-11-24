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
 <a href="https://github.com/concoct-rs/concoct/tree/main/concoct_examples">Examples</a>
</div>

Concoct is an incremental computation framework for Rust.
This library provides a generic diffing engine for user-interfaces and other reactive systems.

This crate is inspired by React, [xilem](https://github.com/linebender/xilem), and [dioxus](https://github.com/dioxuslabs/dioxus).

```rust
fn counter(initial_value: i32) -> impl Composable {
    let mut count = use_state(|| initial_value);

    use_future(|| async move {
        loop {
            time::sleep(Duration::from_millis(500)).await;
            count += 1;
        }
    });

    Debugger::new(count)
}

fn app() -> impl Composable {
    (|| counter(0), || counter(100))
}

#[tokio::main]
async fn main() {
    let mut composition = Composition::new(app);
    composition.build();
    composition.rebuild().await;
    composition.rebuild().await;
}
```