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
 <a href="https://github.com/concoct-rs/concoct/tree/main/web_examples">Examples</a>
</div>

<br />

Concoct is a framework for user-interfaces in Rust.

This crate provides a diffing-engine and state management system for any backend.
Concoct uses static typing to describe your UI at compile-time, which creates an efficient
tree without allocations. Updates to state re-render your application top-down,
starting at the state's parent component.

```rust
struct App;

impl ViewBuilder for App {
    fn build(&self) -> impl View {
        let (count, set_high) = use_state(|| 0);
        let set_low = set_high.clone();

        (
            format!("High five count: {}", count),
            html::button("Up high!").on_click(move |_| set_high(count + 1)),
            html::button("Down low!").on_click(move |_| set_low(count - 1)),
        )
    }
}
```

## Components
```rust
struct Readme {
    crate_name: String,
    version: String,
}

impl ViewBuilder for Readme {
    fn build(&self) -> impl View {
        let (content, set_content) = use_state(|| None);

        use_effect(&self.crate_name, || {
            let name = self.crate_name.clone();
            let version = self.version.clone();
            spawn_local(async move {
                let readme = api::get_readme(&name, &version).await;
                set_content(Some(readme));
            })
        });

        content
            .map(|content| OneOf2::A(content))
            .unwrap_or_else(|| OneOf2::B("Loading..."))
    }
}

struct App;

impl ViewBuilder for App {
    fn build(&self) -> impl View {
        Readme {
            crate_name: String::from("concoct"),
            version: String::from("1.0"),
        }
    }
}
```


## Installation
The easiest way to get started is using the `full` feature flag.

```
cargo add concoct --features full
```

To see a list of the available features flags that can be enabled, check our [docs](https://docs.rs/concoct/latest/concoct/#feature-flags).
