use context::provide_context;

use crate::{
    composable::{
        context,
        text::{TextConfig, TextModifier},
    },
    modify::ModifyExt,
    DevicePixels, Modifier, Modify,
};

pub struct TextStyle {
    pub font_size: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self { font_size: 14.dp() }
    }
}

#[track_caller]
pub fn provide_text_style(text_style: TextStyle, composable: impl FnMut() + 'static) {
    let text_style = if let Some(_last_text_style) = context::<TextStyle>() {
        // TODO merge
        text_style
    } else {
        text_style
    };

    provide_context(text_style, composable);
}

#[track_caller]
pub fn text(modifier: impl Modify<TextConfig> + 'static, string: impl Into<String>) {
    let style = context::<TextStyle>().unwrap_or_default();

    crate::composable::text(Modifier.font_size(style.font_size).chain(modifier), string)
}
