use super::text::{provide_text_style, TextStyle};
use crate::composable::{interaction_source, remember, Container};
use crate::dimension::{DevicePixels, Padding, Size};
use crate::modify::{HandlerModifier, ModifyExt};
use crate::{Composable, Modifier, Modify, View};
use accesskit::Role;
use skia_safe::{Color4f, Point, RGB};
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
/// ```no_run
/// use concoct::View;
/// use concoct::composable::Text;
/// use concoct::composable::material::Button;
///
/// Button::new(|| Text::new( "Press me!"))
///     .on_press(|| {
///         dbg!("Pressed!");
///     }
///     ).view()
/// ```
#[must_use]
pub struct Button<C, F, M> {
    pub content: C,
    pub on_press: F,
    pub modifier: M,
    pub is_enabled: bool,
    pub colors: ButtonColors,
    pub padding: Padding,
    pub size: Size,
}

impl<C> Button<C, (), Modifier> {
    pub fn new(content: C) -> Self {
        Self {
            content,
            on_press: (),
            modifier: Modifier,
            is_enabled: true,
            colors: ButtonColors::new(RGB::from((232, 221, 253)), RGB::from((232, 221, 253))),
            padding: Padding::default().horizontal(Dimension::Points(24.dp())),
            size: Size::default().height(Dimension::Points(40.dp())),
        }
    }
}

impl<C, M, F> Button<C, F, M> {
    pub fn on_press<F2>(self, on_press: F2) -> Button<C, F2, M> {
        Button {
            content: self.content,
            on_press,
            modifier: self.modifier,
            is_enabled: self.is_enabled,
            colors: self.colors,
            padding: self.padding,
            size: self.size,
        }
    }

    pub fn colors(mut self, colors: ButtonColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl<C, F, M> View for Button<C, F, M>
where
    C: FnMut() + 'static,
    F: Composable + 'static,
    M: Modify + 'static,
{
    #[track_caller]
    fn view(self) {
        let color = if self.is_enabled {
            self.colors.enabled
        } else {
            self.colors.disabled
        };

        // TODO this closure fixes an issue with ID's
        let interaction_source = (|| interaction_source())();

        (|| {
            remember([], || {
                interaction_source.on_item(|_interaction| {});
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
                        .clip([Point::new(20.dp(), 20.dp()); 4])
                        .background_color(color)
                        .clickable_interaction(Role::Button, button.on_press, interaction_source)
                        .then(button.modifier),
                )
                .view()
        })
    }
}
