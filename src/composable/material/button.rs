use crate::composable::container;
use crate::modify::container::{ContainerModifier, Padding};
use crate::modify::ModifyExt;
use crate::{modify::container::ContainerConfig, DevicePixels, Modifier, Modify};
use skia_safe::RGB;
use taffy::{
    prelude::Size,
    style::{AlignItems, Dimension, JustifyContent},
};

#[track_caller]
pub fn button(
    modifier: impl Modify<ContainerConfig> + 'static,
    content: impl FnMut() + 'static,
    mut on_press: impl FnMut() + 'static,
) {
    container(
        Modifier
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .merge_descendants()
            .background_color(RGB::from((232, 221, 253)))
            .clickable(move || on_press())
            .padding(Padding::default().horizontal(Dimension::Points(24.dp())))
            .size(Size {
                width: Dimension::Undefined,
                height: Dimension::Points(40.dp()),
            })
            .chain(modifier),
        content,
    )
}
