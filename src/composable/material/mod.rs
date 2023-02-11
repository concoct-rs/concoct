//! Material design composables

use super::{local, provider};
use skia_safe::Color4f;

pub mod button;
pub use button::Button;

mod icon;
pub use icon::icon;

mod navigation_bar;
pub use navigation_bar::{NavigationBar, NavigationBarItem};

mod radio_button;
pub use radio_button::RadioButton;

pub mod text;
pub use text::text;

pub struct LocalContentColor {
    color: Color4f,
}

pub fn local_content_color() -> Color4f {
    local::<LocalContentColor>()
        .map(|rc| rc.color)
        .unwrap_or_else(|| Color4f::new(0., 0., 0., 1.))
}

#[track_caller]
pub fn provide_local_content_color(color: Color4f, composable: impl FnMut() + 'static) {
    provider(color, composable)
}
