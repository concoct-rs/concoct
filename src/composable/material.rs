use super::{container, text};
use crate::{
    modify::{container::ContainerModifier, Padding},
    DevicePixels, Modifier, Modify,
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
            .padding(Padding::default().horizontal(24.dp()))
            .size(Size {
                width: Dimension::Undefined,
                height: 40.dp(),
            })
            .chain(modifier.modify),
        move || text(Modifier::default(), label.clone()),
    )
}
