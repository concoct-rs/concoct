use crate::{modify::Padding, text, Modifier};
use skia_safe::RGB;
use taffy::{prelude::Size, style::Dimension};

#[track_caller]
pub fn button(label: impl Into<String>, mut on_press: impl FnMut() + 'static) {
    text(
        Modifier::default()
            .background_color(RGB::from((255, 0, 0)))
            .clickable(move || on_press())
            .padding(Padding::default().horizontal(Dimension::Points(40.)))
            .size(Size {
                width: Dimension::Undefined,
                height: Dimension::Points(80.),
            }),
        label,
    );
}
