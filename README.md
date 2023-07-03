![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

Modular UI compiler and runtime in rust.
This crate can be used a fully functional GUI library (WIP) or as a framework for [your own GUI library](https://github.com/concoct-rs/concoct/blob/main/examples/tree.rs).

```rust
#[composable]
fn counter(interval: Duration) {
    let count = compose!(remember(move || {
        let count = State::new(0);

        let timer_count = count.clone();
        concoct::spawn(async move {
            loop {
                sleep(interval).await;
                timer_count.update(|count| *count += 1);
            }
        });

        count
    }));

    text(*count.get());
}

#[composable]
fn app() {
    compose!(counter(Duration::from_secs(1)));
    compose!(counter(Duration::from_secs(2)));
}

// ...
```

## Runtime
Composables are defined as:
```rust
pub trait Composable {
    type Output;

    fn compose(self, composer: &mut Composer, changed: u32) -> Self::Output;
}
```
They can be created with the #[composable] attribute macro. Composable functions are only run in their parameters have changed.
Changelists for these parameters are passed down from parent to child in the form of a bitfield `changed` to avoid extra diffing.
If a value is already known to have changed, the child composable will skip storing that parameter and run its function.
However, if no parameters have changed, the composable will skip running its block.

To store parameters, the runtime uses an optimized [gap buffer](https://en.wikipedia.org/wiki/Gap_buffer) with groups.
This enables composables that use parameters from their parents to skip storing data twice.




For example consider the following composable function:
```rust
#[composable]
fn button(label: String, count: i32)
```

This will typically store its `label` and `count` parameters right next to each other in the gap buffer as:
```
... | label: String | count: i32 | ...
```

However, if it's parent component already stored the label, this function will skip it:
```rust
#[composable]
fn button_row(label: String) {
    compose!(button(label, 2));
    compose!(button(label, 3));
}
```
```
    button_row:     button #1:   button #2:
... | label: String | count: i32 | count: i32 | ...
```

## Compiler
The compiler comes in the form of the `#[composable]` attribute macro.
For example:
```rust
#[composable]
fn f() -> i32
// Will become:
fn f() -> impl Composable<Output = i32>
```

### Remember
A more advanced example of the compiler is the built-in `remember` composable function.
This function will store a parameter inside a composable and ensure it never changes.
```rust
#[composable]
pub fn remember<T, F>(f: F) -> T
where
    T: Clone + 'static,
    F: FnOnce() -> T + 'static,
{
    composer.cache(false, f)
}

// Will become:

#[must_use]
pub fn remember<T, F>(f: F) -> impl concoct::Composable<Output = T>
where
    T: Clone + 'static,
    F: FnOnce() -> T + 'static,
{
    #[allow(non_camel_case_types)]
    struct remember_composable<T, F> {
        f: F,
        _marker0: std::marker::PhantomData<T>,
    }
    impl<T, F> concoct::Composable for remember_composable<T, F>
    where
        T: Clone + 'static,
        F: FnOnce() -> T + 'static,
    {
        type Output = T;
        fn compose(
            self,
            composer: &mut concoct::Composer,
            changed: u32,
        ) -> Self::Output {
            ();
            let Self { f, .. } = self;
            composer
                .replaceable_group(
                    std::any::TypeId::of::<remember_composable<T, F>>(),
                    move |composer| { composer.cache(false, f) },
                )
        }
    }
    remember_composable {
        f,
        _marker0: std::marker::PhantomData,
    }
}
```
