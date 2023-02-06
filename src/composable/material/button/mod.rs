use crate::composable::container::{container, ContainerModifier};
use crate::modify::ModifyExt;
use crate::{Modifier, Modify};
use skia_safe::Color4f;
use taffy::style::{AlignItems, JustifyContent};

mod modifier;
pub use modifier::{ButtonColors, ButtonConfig, ButtonModifier};

/// Material You filled button composable
/// * `content`: The composable content to be displayed inside the button
/// * `on_press`: Function called after the button is pressed
///
/// # Screenshots
/// ![screenshots](https://developer.android.com/images/reference/androidx/compose/material3/filled-button.png)
///
/// # Examples
/// ```
/// use concoct::Modifier;
/// use concoct::composable::text;
/// use concoct::composable::material::button;
///
/// button(
///     Modifier,
///     || text(Modifier, "Press me!"),
///     || {
///         dbg!("Pressed!");
///     }
/// );
/// ```
#[track_caller]
pub fn button(
    mut modifier: impl Modify<ButtonConfig>,
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

/// Material You text button composable
/// * `content`: The composable content to be displayed inside the button
/// * `on_press`: Function called after the button is pressed
///
/// # Screenshots
/// ![screenshots](https://developer.android.com/images/reference/androidx/compose/material3/text-button.png)
///
/// # Examples
/// ```
/// use concoct::Modifier;
/// use concoct::composable::text;
/// use concoct::composable::material::text_button;
///
/// text_button(
///     Modifier,
///     || text(Modifier, "Press me!"),
///     || {
///         dbg!("Pressed!");
///     }
/// );
/// ```
#[track_caller]
pub fn text_button(
    modifier: impl Modify<ButtonConfig>,
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
