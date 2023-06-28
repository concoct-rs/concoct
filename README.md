![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

Zero-cost UI framework in Rust.
This crate builds statically allocated state machines with the `composable` macro.

```rust
#[composable]
fn app(count: i32) {
    dbg!(count);
}

fn main() {
    let mut composer = Composer::new();
    composer.compose(app(0)); // Displays 0
    composer.compose(app(0)); // Displays nothing!
    composer.compose(app(1)); // Displays 1
}
```