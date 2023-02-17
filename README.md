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
use concoct::{composable::Text, render::run};

fn app() {
    Text::new("Hello World!")
}

fn main() {
    run(app)
}
```

# Counter
```rust
Container::build_column(|| {
    let count = state(|| 0);

    Text::build(count.get().cloned().to_string())
        .font_size(80.dp())
        .view();

    Container::build_row(move || {
        Button::new(|| Text::new("Less"))
            .on_press(move || *count.get().as_mut() -= 1)
            .view();

        Button::new(|| Text::new("More"))
            .on_press(move || *count.get().as_mut() += 1)
            .view();
    })
    .gap(Size::default().width(Dimension::Points(20.dp())))
    .view()
})
.align_items(AlignItems::Center)
.justify_content(JustifyContent::Center)
.flex_grow(1.)
.gap(Size::default().height(Dimension::Points(20.dp())))
.view()
```

# Creating a composable
To create your own composable, write a function using Rust's `#[track_caller]` attribute macro.
```rust
#[track_caller]
fn title_text(title: String) {
    Text::build(title)
        .font_size(72.dp())
        .modifier(Modifier.clickable(|| {
            dbg!("Click!");
        }))
        .view()
}
```

# State
State is created with the [`state`](https://concoct-rs.github.io/concoct/composable/state/fn.state.html) composable.
```rust
let mut tester = Tester::new(|| {
    Container::new(|| {
        let count = state(|| 0);

        Text::new(count.get().cloned().to_string());

        *count.get().as_mut() += 1;
    })
});

for count in 0..5 {
    assert!(tester.get_text(count.to_string()).is_some());
}
```


