use taffy::{prelude::Size, style::Dimension};

use crate::{container, text, Modifier};

pub fn button(label: impl Into<String>, mut on_press: impl FnMut() + 'static) {
    text(
        Modifier::default()
            .clickable(move |_| on_press())
            .size(Size {
                width: Dimension::Undefined,
                height: Dimension::Points(80.),
            }),
        label,
    );
}
