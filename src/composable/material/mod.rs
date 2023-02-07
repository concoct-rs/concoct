//! Material design composables

use super::{context, provide_context};
use skia_safe::Color4f;

pub mod button;
pub use button::Button;

pub mod icon;
pub use icon::icon;

pub mod radio_button;
pub use radio_button::radio_button;

pub mod text;
pub use text::text;

pub struct LocalContentColor {
    color: Color4f,
}

pub fn local_content_color() -> Color4f {
    context::<LocalContentColor>()
        .map(|rc| rc.color)
        .unwrap_or_else(|| Color4f::new(0., 0., 0., 1.))
}

#[track_caller]
pub fn provide_local_content_color(color: Color4f, composable: impl FnMut() + 'static) {
    provide_context(color, composable)
}
