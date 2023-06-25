![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

Cross-platform UI framework in Rust

This crate is based on Jetpack Compose, the UI framework for Android.

The runtime provides positional memoization to functions marked with the `#[composable]` attribute.
This means that composable functions remember state and can be skipped when their parameters change.
For example, a counter that only prints a `count` when it changes.
```rust
#[composable]
pub fn counter(count: i32) {
    dbg!(count);
}
```
