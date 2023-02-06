use crate::composable::container;
use crate::modify::container::{ContainerModifier, Padding};
use crate::modify::{Chain, ModifyExt};
use crate::{DevicePixels, Modifier, Modify};
use skia_safe::{Color4f, RGB};
use taffy::{
    prelude::Size,
    style::{AlignItems, Dimension, JustifyContent},
};

pub struct ButtonConfig {
    pub is_enabled: bool,
    pub colors: ButtonColors,
    padding: Padding,
    size: Size<Dimension>,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            is_enabled: true,
            colors: ButtonColors::new(RGB::from((232, 221, 253)), RGB::from((232, 221, 253))),
            padding: Padding::default().horizontal(Dimension::Points(24.dp())),
            size: Size {
                width: Dimension::Undefined,
                height: Dimension::Points(40.dp()),
            },
        }
    }
}

impl AsMut<Size<Dimension>> for ButtonConfig {
    fn as_mut(&mut self) -> &mut Size<Dimension> {
        &mut self.size
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

    let color = if config.is_enabled {
        config.colors.enabled
    } else {
        config.colors.disabled
    };

    container(
        Modifier
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .merge_descendants()
            .background_color(color)
            .clickable(on_press)
            .padding(config.padding)
            .size(config.size),
        content,
    )
}

pub trait ButtonModifier: Modify<ButtonConfig> + Sized {
    fn is_enabled(self, is_enabled: bool) -> Chain<ButtonConfig, Self, IsEnabled> {
        self.chain(IsEnabled(is_enabled))
    }

    fn colors(self, button_colors: ButtonColors) -> Chain<ButtonConfig, Self, ButtonColors> {
        self.chain(button_colors)
    }
}

pub struct IsEnabled(bool);

impl Modify<ButtonConfig> for IsEnabled {
    fn modify(&mut self, value: &mut ButtonConfig) {
        value.is_enabled = self.0;
    }
}

#[derive(Clone)]
pub struct ButtonColors {
    enabled: Color4f,
    disabled: Color4f,
}

impl ButtonColors {
    pub fn new(enabled: impl Into<Color4f>, disabled: impl Into<Color4f>) -> Self {
        Self {
            enabled: enabled.into(),
            disabled: disabled.into(),
        }
    }

    pub fn from_color(color: impl Into<Color4f>) -> Self {
        Self::from(color.into())
    }
}

impl From<Color4f> for ButtonColors {
    fn from(value: Color4f) -> Self {
        Self::new(value, value)
    }
}

impl Modify<ButtonConfig> for ButtonColors {
    fn modify(&mut self, value: &mut ButtonConfig) {
        value.colors = self.clone();
    }
}
