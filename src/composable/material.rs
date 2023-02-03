use super::{container, text};
use crate::{modify::Padding, Modifier};
use skia_safe::RGB;
use taffy::{
    prelude::Size,
    style::{AlignItems, Dimension},
};

#[track_caller]
pub fn button(label: impl Into<String>, mut on_press: impl FnMut() + 'static) {
    let label = label.into();
    container(
        Modifier::default()
            .align_items(AlignItems::Center)
            .merge_descendants()
            .background_color(RGB::from((255, 0, 0)))
            .clickable(move || on_press())
            .padding(Padding::default().horizontal(Dimension::Points(40.)))
            .size(Size {
                width: Dimension::Undefined,
                height: Dimension::Points(80.),
            }),
        move || text(Modifier::default(), label.clone()),
    )
}
