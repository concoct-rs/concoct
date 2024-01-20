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
struct App;

impl View for App {
    fn body(&self) -> impl Body {
        let (count, set_count_high) = use_state(|| 0);
        let set_count_low = set_count_high.clone();

        let n = *count;
        (
            format!("High five count: {}", count),
            html::button("Up high!").on_click(move |_| set_count_high(n + 1)),
            html::button("Down low!").on_click(move |_| set_count_low(n - 1)),
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
