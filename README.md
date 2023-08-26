![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://concoct-rs.github.io/concoct/)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

[Examples](https://github.com/concoct-rs/concoct/tree/main/examples)

Rust zero-cost reactive UI framework.

## Features
 - Inspired by the elm architecture
 - Compile-time UI tree

```rust
enum Message {
    Increment,
    Decrement,
}

fn counter(count: &i32) -> impl View<Message> {
    (
        h1([], count.to_string()),
        button([on("click", Message::Increment)], "More"),
        button([on("click", Message::Decrement)], "Less"),
    )
}

fn main() {
    concoct::run(
        0,
        |count, msg| match msg {
            Message::Increment => *count += 1,
            Message::Decrement => *count -= 1,
        },
        counter,
    );
}
```

