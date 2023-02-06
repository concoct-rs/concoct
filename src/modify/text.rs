use super::{Chain, ModifyExt};
use crate::{composable::text::TextConfig, Modify};
use skia_safe::Typeface;

pub trait TextModifier {
    type Modify;

    fn font_size(self, value: f32) -> Chain<Self::Modify, FontSize>;

    fn typeface(self, typeface: Typeface) -> Chain<Self::Modify, Typeface>;
}

impl<M: Modify<TextConfig>> TextModifier for M {
    type Modify = M;

    fn font_size(self, value: f32) -> Chain<Self::Modify, FontSize> {
        self.chain(FontSize { value })
    }

    fn typeface(self, typeface: Typeface) -> Chain<Self::Modify, Typeface> {
        self.chain(typeface)
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
