use crate::composable::container;
use crate::modify::container::ContainerModifier;
use crate::modify::ModifyExt;
use crate::{Modifier, Modify};
use skia_safe::Color4f;
use taffy::style::{AlignItems, JustifyContent};

mod modifier;
pub use modifier::{ButtonColors, ButtonConfig, ButtonModifier};

/// Material You filled button
#[track_caller]
pub fn button(
    mut modifier: impl Modify<ButtonConfig> + 'static,
    content: impl FnMut() + 'static,
    on_press: impl FnMut() + 'static,
) {
    let mut config = ButtonConfig::default();
    modifier.modify(&mut config);

    let color = if config.is_enabled {
        config.colors.enabled
    } else {
        config.colors.disabled
    };

    container(
        Modifier
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .merge_descendants()
            .background_color(color)
            .clickable(on_press)
            .padding(config.padding)
            .size(config.size),
        content,
    )
}

/// Material You filled button
#[track_caller]
pub fn text_button(
    modifier: impl Modify<ButtonConfig> + 'static,
    content: impl FnMut() + 'static,
    on_press: impl FnMut() + 'static,
) {
    button(
        Modifier
            .colors(ButtonColors::from_color(Color4f::new(0., 0., 0., 0.)))
            .chain(modifier),
        content,
        on_press,
    )
}
