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
 <a href="https://github.com/concoct-rs/concoct/tree/main/web_examples">Examples</a>
</div>

<br />

Concoct is a framework for user-interfaces in Rust.

This crate provides a diffing-engine and state management system for any backend.
Concoct uses static typing to describe your UI at compile-time to create an efficient
tree without allocations. Updates to state re-render your application top-down,
starting at the state's parent component.

```rust
#[derive(Default)]
struct Counter {
    count: i32,
}

impl View<Counter> for Counter {
    fn body(&mut self, _cx: &Scope<Counter>) -> impl View<Counter> {
        (
            format!("High five count: {}", self.count),
            html::button("Up high!").on_click(|state: &mut Self, _event| state.count += 1),
            html::button("Down low!").on_click(|state: &mut Self, _event| state.count -= 1),
        )
    }
}
```

## Installation
The easiest way to get started is using the `full` feature flag.

```
cargo add concoct --features full
```

To see a list of the available features flags that can be enabled, check our [docs](https://docs.rs/concoct/latest/concoct/#feature-flags).

## Inspiration
This crate is inspired by [xilem](https://github.com/linebender/xilem), React, and SwiftUI.
