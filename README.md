![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://concoct-rs.github.io/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

Cross-platform UI framework in Rust with
* Easy functional composasbles
* Flexible state management
* Desktop and mobile support
* Accessibility
* High quality skia rendering

![wallet example](https://github.com/matthunz/viewbuilder/blob/main/screenshots/wallet.png?raw=true)
![counter example](https://github.com/matthunz/viewbuilder/blob/main/screenshots/counter.png?raw=true)

# Getting started
```rust
use concoct::{composable::text, render::run, Modifier};

fn app() {
    text(Modifier::default(), "Hello World!")
}

fn main() {
    run(app)
}
```
