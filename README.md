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
 <a href="https://github.com/concoct-rs/concoct/tree/main/examples">Native Examples</a>
  |
 <a href="https://github.com/concoct-rs/concoct/tree/main/web_examples">Web Examples</a>
</div>

<br />

Concoct is a runtime for user-interfaces in Rust.

```rust
use concoct::{Handler, Runtime, Task};

#[derive(Default)]
pub struct Counter {
    value: i32,
}

impl Task for Counter {}

impl Handler<i32> for Counter {
    fn handle(&mut self, msg: i32) {
        self.value = msg;
    }
}

#[tokio::main]
async fn main() {
    let rt = Runtime::default();
    let _guard = rt.enter();

    let a = Counter::default().spawn();
    let b = Counter::default().spawn();

    a.bind(&b);

    a.send(1);
    a.send(2);

    rt.run().await;

    assert_eq!(a.borrow().value, 2);
    assert_eq!(b.borrow().value, 2);
}
```
