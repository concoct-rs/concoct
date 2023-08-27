![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/crate/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

[Examples](https://github.com/concoct-rs/concoct/tree/main/examples)

Rust zero-cost reactive UI framework.

## Features
 - Inspired by the elm architecture
 - Compile-time UI tree

```rust
enum Event {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Event> {
    (
        h1(count.to_string()),
        button("More").modify(on("click", || Event::Increment)),
        button("Less").modify(on("click", || Event::Decrement)),
    )
}

fn main() {
    concoct::run(
        0,
        |count, event| match event {
            Event::Increment => *count += 1,
            Event::Decrement => *count -= 1,
        },
        counter,
    );
}
```

