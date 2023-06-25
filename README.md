![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

Cross-platform UI framework in Rust.
This crate is based on Jetpack Compose, the UI framework for Android.

## Runtime
The runtime provides positional memoization to functions marked with the `#[composable]` attribute.
This means that composable functions remember state and can be skipped when their parameters change.
For example, a counter that only prints a `count` when it changes:
```rust
#[composable]
pub fn counter(count: i32) {
    dbg!(count);
}

counter(1); // Displays 1
counter(1); // Displays nothing
counter(2); // Displays 2
```

### Compiler
The runtime is heavily optimized to make use of the compiler.
When creating a `#[composable]` function, the compiler transforms it to use the runtime.
The previous example can be compiled as: 
```rust
pub fn counter(count: i32) {
    panic!("Must be called from a concoct runtime.")
}

fn counterComposable(composer: &mut impl concoct::Compose, changed: u64, count: i32) {
    composer.start_restart_group(0u64);
    let mut dirty = changed;
    if changed & 14u64 == 0 {
        dirty = changed | if composer.changed(&count) { 4 } else { 2 };
    }
    if dirty & 11u64 == 2 && composer.is_skipping() {
        composer.skip_to_group_end();
    } else {
        dbg!(count);
    }
    composer.end_restart_group(|composer| counterComposable(composer, changed | 1, count));
}
```
