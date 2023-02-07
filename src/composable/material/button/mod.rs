use super::text::{provide_text_style, TextStyle};
use crate::composable::container::modifier::{ContainerConfig, Padding};
use crate::composable::container::{container, ContainerModifier};
use crate::composable::{interaction_source, remember};
use crate::modify::{HandlerModifier, ModifyExt};
use crate::{DevicePixels, Modifier, Modify};
use accesskit::Role;
use skia_safe::{Color4f, RGB};
use taffy::prelude::Size;
use taffy::style::{AlignItems, Dimension, JustifyContent};

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
#[must_use = "Buttons must be viewed with `Button::view`"]
pub struct Button<C, M, F> {
    pub content: C,
    pub modifier: M,
    pub on_press: F,
    pub is_enabled: bool,
    pub colors: ButtonColors,
    pub padding: Padding,
    pub size: Size<Dimension>,
}

impl<C, F> Button<C, Modifier, F> {
    pub fn build(content: C, on_press: F) -> Self {
        Self {
            content,
            on_press,
            modifier: Modifier,
            is_enabled: true,
            colors: ButtonColors::new(RGB::from((232, 221, 253)), RGB::from((232, 221, 253))),
            padding: Padding::default().horizontal(Dimension::Points(24.dp())),
            size: Size {
                width: Dimension::Undefined,
                height: Dimension::Points(40.dp()),
            },
        }
    }

    #[track_caller]
    pub fn new(content: C, on_press: F)
    where
        C: FnMut() + Clone + 'static,
        F: FnMut() + Clone + 'static,
    {
        Self::build(content, on_press).view();
    }
}

impl<C, M, F> Button<C, M, F>
where
    C: FnMut() + Clone + 'static,
    M: Modify<ContainerConfig> + 'static,
    F: FnMut() + Clone + 'static,
{
    #[track_caller]
    pub fn view(self) {
        let color = if self.is_enabled {
            self.colors.enabled
        } else {
            self.colors.disabled
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

        let mut modifier = Some(self.modifier);

        provide_text_style(text_style, move || {
            container(
                Modifier
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .merge_descendants()
                    .background_color(color)
                    .clickable_interaction(Role::Button, self.on_press.clone(), interaction_source)
                    .padding(self.padding)
                    .size(self.size)
                    .chain(modifier.take().unwrap()),
                self.content.clone(),
            )
        })
    }
}
