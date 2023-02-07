![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://concoct-rs.github.io/concoct)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

Cross-platform UI framework in Rust with
* Easy functional composasbles
* Flexible state management
* Desktop and mobile support
* Accessibility
* Native skia rendering

[Examples](https://github.com/concoct-rs/concoct/tree/main/examples)

![wallet example](https://github.com/matthunz/viewbuilder/blob/main/screenshots/wallet.png?raw=true)
![counter example](https://github.com/matthunz/viewbuilder/blob/main/screenshots/counter.png?raw=true)

# Hello World
```rust
use concoct::{composable::text, render::run, Modifier};

fn app() {
    Text::new( "Hello World!")
}

fn main() {
    run(app)
}
```

# Creating a composable
To create your own composable, write a function using Rust's `#[track_caller]` attribute macro.
```rust
#[track_caller]
fn title_text(title: String) {
    text(Modifier.font_size(80.dp()), title);
}
```

# State
State is created with the [`state`](https://concoct-rs.github.io/concoct/composable/state/fn.state.html) composable.
```rust
let mut tester = Tester::new(|| {
    container(Modifier, || {
        let count = state(|| 0);

        Text::new( count.get().cloned().to_string());

        *count.get().as_mut() += 1;
    })
});

for count in 0..5 {
    assert!(tester.get_text(count.to_string()).is_some());
}
```


