use super::{Chain, ModifyExt};
use crate::{composable::text::TextConfig, Modify};
use skia_safe::Typeface;

pub trait TextModifier: Modify<TextConfig> + Sized {
    fn font_size(self, value: f32) -> Chain<TextConfig, Self, FontSize> {
        self.chain(FontSize { value })
    }

    fn typeface(self, typeface: Typeface) -> Chain<TextConfig, Self, Typeface> {
        self.chain(typeface)
    }
}

impl<M: Modify<TextConfig>> TextModifier for M {}

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
