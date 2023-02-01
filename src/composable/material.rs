use taffy::{prelude::Size, style::Dimension};

use crate::{text, Modifier};

#[track_caller]
pub fn button(label: impl Into<String>, mut on_press: impl FnMut() + 'static) {
    text(
        Modifier::default()
            .clickable(move || on_press())
            .size(Size {
                width: Dimension::Undefined,
                height: Dimension::Points(80.),
            }),
        label,
    );
}
