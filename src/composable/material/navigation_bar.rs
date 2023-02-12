use super::{
    provide_local_content_color,
    text::{provide_text_style, TextStyle},
};
use crate::{
    composable::{
        container::{Gap, Padding},
        Container,
    },
    modify::{HandlerModifier, ModifyExt},
    DevicePixels, Modifier, Modify, View,
};
use accesskit::Role;
use skia_safe::{Color4f, RGB};
use taffy::{
    prelude::{Dimension, Size},
    style::{AlignItems, JustifyContent},
};

#[must_use]
pub struct NavigationBar<C, M> {
    content: C,
    modifier: M,
    container_color: Color4f,
    content_color: Color4f,
}

impl<C> NavigationBar<C, Modifier> {
    pub fn build(content: C) -> Self {
        Self {
            content,
            modifier: Modifier,
            container_color: Color4f::from(RGB::from((242, 237, 246))),
            content_color: Color4f::from(RGB::from((0, 0, 0))),
        }
    }

    #[track_caller]
    pub fn new(content: C)
    where
        C: FnMut() + 'static,
    {
        Self::build(content).view()
    }
}

impl<C, M> NavigationBar<C, M> {
    pub fn modifier<M2>(self, modifier: M2) -> NavigationBar<C, M2> {
        NavigationBar {
            content: self.content,
            modifier,
            container_color: self.container_color,
            content_color: self.content_color,
        }
    }
}

impl<C, M> View for NavigationBar<C, M>
where
    C: FnMut() + 'static,
    M: Modify + 'static,
{
    #[track_caller]
    fn view(self) {
        let mut content_cell = Some(self.content);

        Container::build_row(move || {
            provide_local_content_color(self.content_color, content_cell.take().unwrap())
        })
        .flex_shrink(0.)
        .gap(Gap::default().width(Dimension::Points(16.dp())))
        .padding(
            Padding::default()
                .top(Dimension::Points(12.dp()))
                .bottom(Dimension::Points(16.dp()))
                .horizontal(Dimension::Points(8.dp())),
        )
        .size(Size {
            width: Dimension::Percent(1.),
            height: Dimension::Undefined,
        })
        .modifier(
            Modifier
                .background_color(self.container_color)
                .chain(self.modifier),
        )
        .view()
    }
}

#[must_use]
pub struct NavigationBarItem<I, L, M, F> {
    icon: I,
    label: L,
    modifier: M,
    on_click: F,
    is_selected: bool,
}

impl<I, L, F> NavigationBarItem<I, L, Modifier, F> {
    pub fn build(icon: I, label: L, on_click: F) -> Self {
        Self {
            icon,
            label,
            modifier: Modifier,
            on_click,
            is_selected: false,
        }
    }

    #[track_caller]
    pub fn new(icon: I, label: L, on_click: F)
    where
        I: FnMut() + 'static,
        L: FnMut() + 'static,
        F: FnMut() + 'static,
    {
        Self::build(icon, label, on_click).view()
    }
}

impl<I, L, M, F> NavigationBarItem<I, L, M, F> {
    pub fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }
}

impl<I, L, M, F> View for NavigationBarItem<I, L, M, F>
where
    I: FnMut() + 'static,
    L: FnMut() + 'static,
    M: Modify + 'static,
    F: FnMut() + 'static,
{
    #[track_caller]
    fn view(self) {
        let mut icon_cell = Some(self.icon);
        let mut label_cell = Some(self.label);

        let icon_background_color = if self.is_selected {
            RGB::from((232, 221, 253)).into()
        } else {
            Color4f::new(0., 0., 0., 0.)
        };

        Container::build_column(move || {
            let mut icon_cell = icon_cell.take();

            Container::build_column(move || {
                icon_cell.take().unwrap()();
            })
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .size(Size {
                width: Dimension::Percent(1.),
                height: Dimension::Points(32.dp()),
            })
            .modifier(Modifier.background_color(icon_background_color))
            .view();

            provide_text_style(TextStyle { font_size: 12.dp() }, label_cell.take().unwrap());
        })
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .gap(Gap::default().height(Dimension::Points(4.dp())))
        .size(Size {
            width: Dimension::Percent(1.),
            height: Dimension::Percent(1.),
        })
        .modifier(
            Modifier
                .clickable(Role::Navigation, self.on_click)
                .chain(self.modifier),
        )
        .view()
    }
}
