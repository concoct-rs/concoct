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

Concoct is a runtime for user-interfaces in Rust.

This crate provides an event-driven state management system that runs anywhere (including `#![no_std]`).

```rust
use concoct::{Context, Object, Signal};

#[derive(Default)]
struct Counter {
    value: i32,
}

impl Object for Counter {}

impl Signal<i32> for Counter {}

impl Counter {
    fn set_value(cx: &mut Context<Self>, value: i32) {
        if cx.value != value {
            cx.value = value;
            cx.emit(value);
        }
    }
}

fn main() {
    let a = Counter::create();
    let b = Counter::create();

    a.bind(&b, Counter::set_value);

    Counter::set_value(&mut a.cx(), 2);

    assert_eq!(a.borrow().value, 2);
    assert_eq!(b.borrow().value, 2);
}
```

## Installation
The easiest way to get started is using the `full` feature flag.

```
cargo add concoct --features full
```

To see a list of the available features flags that can be enabled, check our [docs](https://docs.rs/concoct/latest/concoct/#feature-flags).
