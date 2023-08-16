![Concoct](https://github.com/matthunz/viewbuilder/blob/main/logo.png?raw=true)

[![crate](https://img.shields.io/crates/v/concoct.svg)](https://crates.io/crates/concoct)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://concoct-rs.github.io/concoct/)
[![CI](https://github.com/matthunz/concoct/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/concoct/actions/workflows/rust.yml)

[Examples](https://github.com/concoct-rs/concoct/tree/main/examples)

Rust native UI framework.

## Features
- Statically typed, compile-time UI tree
- Accessability with [accesskit](https://github.com/AccessKit/accesskit)
- Flexbox layout with [taffy](https://github.com/DioxusLabs/taffy)
- Inspired by the [xilem](https://github.com/linebender/xilem) and elm architectures.

```rust
fn circle(radius: f32) -> impl View<f32> {
    Canvas::new(move |_layout, canvas| {
        let color = Color4f::new(1., 0., 0., 1.);
        canvas.draw_circle((radius, radius), radius, &Paint::new(color, None));
    })
    .size(Size::from_points(radius * 2., radius * 2.))
}

fn app() -> impl View<()> {
    remember(
        || 50.,
        |radius: &mut f32| clickable(Role::Button, |r: &mut f32| *r *= 2., circle(*radius)),
    )
}
```

