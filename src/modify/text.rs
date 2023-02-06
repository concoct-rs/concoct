use skia_safe::Typeface;

use crate::{composable::text::TextConfig, Modifier, Modify};

use super::Chain;

pub trait TextModifier {
    type Modify;

    fn font_size(self, value: f32) -> Modifier<TextConfig, Chain<Self::Modify, FontSize>>;

    fn typeface(self, typeface: Typeface) -> Modifier<TextConfig, Chain<Self::Modify, Typeface>>;
}

impl<M> TextModifier for Modifier<TextConfig, M> {
    type Modify = M;

    fn font_size(self, value: f32) -> Modifier<TextConfig, Chain<Self::Modify, FontSize>> {
        self.chain(FontSize { value })
    }

    fn typeface(self, typeface: Typeface) -> Modifier<TextConfig, Chain<Self::Modify, Typeface>> {
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
