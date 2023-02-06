use super::text::{provide_text_style, TextStyle};
use crate::composable::container::{container, ContainerModifier};
use crate::composable::{interaction_source, remember};
use crate::modify::{HandlerModifier, ModifyExt};
use crate::{DevicePixels, Modifier, Modify};
use accesskit::Role;
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
    content: impl FnMut() + Clone + 'static,
    on_press: impl FnMut() + Clone + 'static,
) {
    let mut config = ButtonConfig::default();
    modifier.modify(&mut config);

    let color = if config.is_enabled {
        config.colors.enabled
    } else {
        config.colors.disabled
    };

    // TODO this closure fixes an issue with ID's
    let interaction_source = (|| interaction_source())();

    (|| {
        remember([], || {
            interaction_source.on_item(|interaction| {
                dbg!(interaction);
            });
        })
    })();

    let mut text_style = TextStyle::default();
    text_style.font_size = 18.dp();

    provide_text_style(text_style, move || {
        container(
            Modifier
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .merge_descendants()
                .background_color(color)
                .clickable_interaction(Role::Button, on_press.clone(), interaction_source)
                .padding(config.padding)
                .size(config.size),
            content.clone(),
        )
    })
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
    content: impl FnMut() + Clone + 'static,
    on_press: impl FnMut() + Clone + 'static,
) {
    button(
        Modifier
            .colors(ButtonColors::from_color(Color4f::new(0., 0., 0., 0.)))
            .chain(modifier),
        content,
        on_press,
    )
}
