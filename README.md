![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

Cross-platform UI runtime and framework in Rust.

```rust
use concoct::{composable, compose};

#[composable]
fn counter(count: i32) {
    dbg!(count);
}

#[composable]
fn app(x: i32, y: i32) {
    compose!(counter(x));

    compose!(counter(y));
}
```