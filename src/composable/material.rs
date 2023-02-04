use super::{container, text};
use crate::{
    modify::{container::ContainerModifier, Padding},
    Modifier, Modify,
};
use skia_safe::RGB;
use taffy::{
    prelude::Size,
    style::{AlignItems, Dimension, JustifyContent},
};

#[track_caller]
pub fn button(
    modifier: Modifier<ContainerModifier, impl Modify<ContainerModifier> + 'static>,
    label: impl Into<String>,
    mut on_press: impl FnMut() + 'static,
) {
    let label = label.into();
    container(
        Modifier::default()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .merge_descendants()
            .background_color(RGB::from((232, 221, 253)))
            .clickable(move || on_press())
            .padding(Padding::default().horizontal(Dimension::Points(40.)))
            .size(Size {
                width: Dimension::Undefined,
                height: Dimension::Points(80.),
            })
            .chain(modifier.modify),
        move || text(Modifier::default(), label.clone()),
    )
}
