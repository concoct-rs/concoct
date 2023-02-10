use super::provide_local_content_color;
use crate::{
    composable::{
        container::{Gap, Padding},
        Container,
    },
    modify::ModifyExt,
    DevicePixels, Modifier, Modify,
};
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
    #[track_caller]
    pub fn view(self)
    where
        C: FnMut() + 'static,
        M: Modify<()> + 'static,
    {
        let mut content_cell = Some(self.content);

        Container::build_row(move || {
            provide_local_content_color(self.content_color, content_cell.take().unwrap())
        })
        .padding(
            Padding::default()
                .top(Dimension::Points(12.dp()))
                .bottom(Dimension::Points(16.dp())),
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
pub struct NavigationBarItem<I, L, M> {
    icon: I,
    label: L,
    modifier: M,
}

impl<I, L> NavigationBarItem<I, L, Modifier> {
    pub fn build(icon: I, label: L) -> Self {
        Self {
            icon,
            label,
            modifier: Modifier,
        }
    }

    #[track_caller]
    pub fn new(icon: I, label: L)
    where
        I: FnMut() + 'static,
        L: FnMut() + 'static,
    {
        Self::build(icon, label).view()
    }
}

impl<I, L, M> NavigationBarItem<I, L, M> {
    #[track_caller]
    pub fn view(mut self)
    where
        I: FnMut() + 'static,
        L: FnMut() + 'static,
        M: Modify<()> + 'static,
    {
        let mut icon_cell = Some(self.icon);

        Container::build_column(move || {
            let mut icon_cell = icon_cell.take();

            Container::build_column(move || {
                icon_cell.take().unwrap()();
            })
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .size(Size {
                width: Dimension::Percent(1.),
                height: Dimension::Points(32.dp()),
            })
            .view();

            (self.label)()
        })
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .gap(Gap::default().height(Dimension::Points(4.dp())))
        .size(Size {
            width: Dimension::Percent(1.),
            height: Dimension::Percent(1.),
        })
        .view()
    }
}
