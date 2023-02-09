use local::provider;

use super::local_content_color;
use crate::{composable::local, DevicePixels, Modify};

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
    let text_style = if let Some(_last_text_style) = local::<TextStyle>() {
        // TODO merge
        text_style
    } else {
        text_style
    };

    provider(text_style, composable);
}

pub struct Text {}

#[track_caller]
pub fn text(modifier: impl Modify<()> + 'static, string: impl Into<String>) {
    let style = local::<TextStyle>().unwrap_or_default();
    let color = local_content_color();

    crate::composable::Text::build(string)
        .color(color)
        .font_size(style.font_size)
        .modifier(modifier)
        .view()
}
