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

<br />

Concoct is a reactive runtime for embedded systems.

```rust
use concoct::{
    task::{self, Task},
    System,
};

fn app(_count: &mut i32) -> impl Task<i32> {
    task::from_fn(|_| dbg!("Hello World!"))
}

fn main() {
    let mut system = System::new(0, app);
    system.build();
    system.rebuild();
}
```

## Inspiration
This crate is inspired by [xilem](https://github.com/linebender/xilem), [Drake](https://drake.mit.edu) and [ArduPilot](https://ardupilot.org).
