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
 <a href="https://github.com/concoct-rs/concoct/tree/main/examples">Native Examples</a>
  |
 <a href="https://github.com/concoct-rs/concoct/tree/main/web_examples">Web Examples</a>
</div>

<br />

Concoct is an incremental computation framework for Rust.
This library provides a generic diffing engine for user-interfaces and other reactive systems.

This crate is inspired by Jetpack Compose, [xilem](https://github.com/linebender/xilem), and [dioxus](https://github.com/dioxuslabs/dioxus).

```rust
use concoct::prelude::*;

#[derive(PartialEq)]
struct Counter {
    initial_value: i32,
}

impl View for Counter {
    fn view(&mut self) -> impl IntoView {
        let mut count = use_state(|| self.initial_value);

        (
            format!("High five count: {count}"),
            button("Up high").on_click(move || count += 1),
            button("Down low").on_click(move || count -= 1),
        )
    }
}

fn main() {
    concoct::web::run(Counter { initial_value: 0 })
}
```

## Installation
This crate currently requires rust nightly.
You can install concoct for web by running:
```
cargo add concoct --features web
```
