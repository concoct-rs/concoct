use crate::{
    composable::text::TextConfig,
    modify::{Chain, ModifyExt},
    Modify,
};
use skia_safe::{Color4f, Typeface};

pub trait TextModifier: Modify<TextConfig> + Sized {
    fn color(self, color: impl Into<Color4f>) -> Chain<TextConfig, Self, Color4f> {
        self.chain(color.into())
    }

    fn font_size(self, value: f32) -> Chain<TextConfig, Self, FontSize> {
        self.chain(FontSize { value })
    }

    fn typeface(self, typeface: Typeface) -> Chain<TextConfig, Self, Typeface> {
        self.chain(typeface)
    }
}

impl<M: Modify<TextConfig>> TextModifier for M {}

impl Modify<TextConfig> for Color4f {
    fn modify(&mut self, value: &mut TextConfig) {
        value.color = *self;
    }
}

impl Modify<TextConfig> for Typeface {
    fn modify(&mut self, value: &mut TextConfig) {
        value.typeface = self.clone();
    }
}

pub struct FontSize {
    value: f32,
}

impl Modify<TextConfig> for FontSize {
    fn modify(&mut self, value: &mut TextConfig) {
        value.font_size = self.value;
    }
}
