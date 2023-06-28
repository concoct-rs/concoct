![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

Zero-cost UI framework in Rust.
This crate builds statically allocated state machines with the `composable` macro.

```rust
#![feature(type_alias_impl_trait)]

use concoct::{composable, compose, Composer};

#[composable]
fn counter(count: i32) {
    dbg!(count);
}

#[composable]
fn app(x: i32, y: i32) {
    compose!(counter(x));

    compose!(counter(y));
}

fn main() {
    let mut composer = Composer::default();
    composer.compose(app(0, 0)); // 0, 0
    composer.compose(app(0, 0)); // Displays nothing!

    composer.compose(app(1, 0)); // 1
    composer.compose(app(1, 0)); // Displays nothing!

    composer.compose(app(0, 1)); // 0, 1
    composer.compose(app(0, 1)); // Displays nothing!
}
```