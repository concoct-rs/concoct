use crate::composable::container::modifier::Padding;
use crate::modify::{Chain, ModifyExt};
use crate::{DevicePixels, Modify};
use skia_safe::{Color4f, RGB};
use taffy::{prelude::Size, style::Dimension};

pub struct ButtonConfig {
    pub is_enabled: bool,
    pub colors: ButtonColors,
    pub padding: Padding,
    pub size: Size<Dimension>,
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

pub trait ButtonModifier: Modify<ButtonConfig> + Sized {
    fn is_enabled(self, is_enabled: bool) -> Chain<ButtonConfig, Self, IsEnabled> {
        self.chain(IsEnabled(is_enabled))
    }

    fn colors(self, button_colors: ButtonColors) -> Chain<ButtonConfig, Self, ButtonColors> {
        self.chain(button_colors)
    }
}

impl<M: Modify<ButtonConfig>> ButtonModifier for M {}

pub struct IsEnabled(bool);

impl Modify<ButtonConfig> for IsEnabled {
    fn modify(&mut self, value: &mut ButtonConfig) {
        value.is_enabled = self.0;
    }
}

#[derive(Clone)]
pub struct ButtonColors {
    pub enabled: Color4f,
    pub disabled: Color4f,
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
