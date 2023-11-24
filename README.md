<p align="center">
  <img alt="logo" src="./logo.png">
</p>

<div align="center">
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
 <a href="https://github.com/concoct-rs/concoct/tree/main/concoct_examples">Examples</a>
</div>

## An incremental computation framework for Rust.

```rust
fn counter() -> impl Composable {
    let mut count = use_state(|| 0);

    use_future(|| async move {
        loop {
            count += 1;
            time::sleep(Duration::from_millis(500)).await;
        }
    });

    Debugger::new(count)
}

fn main() {
    let mut composition = Composition::new(counter);
    composition.build();
    composition.rebuild();
}
```