use crate::composable::container;
use crate::modify::container::{ContainerModifier, Padding};
use crate::modify::ModifyExt;
use crate::{DevicePixels, Modifier, Modify};
use skia_safe::RGB;
use taffy::{
    prelude::Size,
    style::{AlignItems, Dimension, JustifyContent},
};

pub struct ButtonConfig {
    padding: Padding,
    size: Size<Dimension>,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            padding: Padding::default().horizontal(Dimension::Points(24.dp())),
            size: Size {
                width: Dimension::Undefined,
                height: Dimension::Points(40.dp()),
            },
        }
    }
}

/// Material You filled button
#[track_caller]
pub fn button(
    mut modifier: impl Modify<ButtonConfig> + 'static,
    content: impl FnMut() + 'static,
    on_press: impl FnMut() + 'static,
) {
    let mut config = ButtonConfig::default();
    modifier.modify(&mut config);

    container(
        Modifier
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .merge_descendants()
            .background_color(RGB::from((232, 221, 253)))
            .clickable(on_press)
            .padding(config.padding)
            .size(config.size),
        content,
    )
}
