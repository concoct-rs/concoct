//! Material design composables

pub mod button;
use std::rc::Rc;

pub use button::{button, text_button};

pub mod icon;
pub use icon::icon;

pub mod text;
use skia_safe::Color4f;
pub use text::text;

use super::{context, provide_context};

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
