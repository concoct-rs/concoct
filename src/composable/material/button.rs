use super::text::{provide_text_style, TextStyle};
use crate::composable::container::Padding;
use crate::composable::{interaction_source, remember, Container};
use crate::modify::{HandlerModifier, ModifyExt};
use crate::{DevicePixels, Modifier, Modify};
use accesskit::Role;
use skia_safe::{Color4f, RGB};
use taffy::prelude::Size;
use taffy::style::{AlignItems, Dimension, JustifyContent};

#[derive(Clone)]
pub struct ButtonColors {
    pub enabled: Color4f,
    pub disabled: Color4f,
}

impl ButtonColors {
    pub fn new(enabled: impl Into<Color4f>, disabled: impl Into<Color4f>) -> Self {
        Self {
            enabled: enabled.into(),
            disabled: disabled.into(),
        }
    }

    pub fn from_color(color: impl Into<Color4f>) -> Self {
        Self::from(color.into())
    }
}

impl From<Color4f> for ButtonColors {
    fn from(value: Color4f) -> Self {
        Self::new(value, value)
    }
}

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
/// use concoct::composable::Text;
/// use concoct::composable::material::Button;
///
/// Button::new(
///     || {
///         dbg!("Pressed!");
///     },
///     || Text::new( "Press me!"),
/// )
/// ```
#[must_use = "Buttons must be viewed with `Button::view`"]
pub struct Button<C, F, M> {
    pub content: C,
    pub on_press: F,
    pub modifier: M,
    pub is_enabled: bool,
    pub colors: ButtonColors,
    pub padding: Padding,
    pub size: Size<Dimension>,
}

impl<C, F> Button<C, F, Modifier> {
    pub fn build(on_press: F, content: C) -> Self {
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
    pub fn new(on_press: F, content: C)
    where
        C: FnMut() + Clone + 'static,
        F: FnMut() + Clone + 'static,
    {
        Self::build(on_press, content).view();
    }
}

impl<C, M, F> Button<C, F, M>
where
    C: FnMut() + 'static,
    F: FnMut() + 'static,
    M: Modify<()> + 'static,
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

        let mut cell = Some(self);
        provide_text_style(text_style, move || {
            let button = cell.take().unwrap();

            Container::build(button.content, Role::Button)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .padding(button.padding)
                .size(button.size)
                .merge_descendants()
                .modifier(
                    Modifier
                        .background_color(color)
                        .clickable_interaction(Role::Button, button.on_press, interaction_source)
                        .chain(button.modifier),
                )
                .view()
        })
    }
}
