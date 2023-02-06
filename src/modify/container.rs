use super::{
    keyboard_input::{KeyboardHandler, KeyboardInput},
    Chain, FlexBasis, FlexGrow, FlexShrink, Margin, ModifyExt,
};
use crate::Modify;
use accesskit::Role;
use taffy::{
    prelude::{Rect, Size},
    style::{AlignItems, Dimension, FlexDirection, JustifyContent, Style},
};

pub struct ContainerConfig {
    pub merge_descendants: bool,
    pub role: Role,
    pub style: Style,
}

pub trait ContainerModifier: Modify<ContainerConfig> + Sized {
    fn merge_descendants(self) -> Chain<Self, MergeDescendants> {
        self.chain(MergeDescendants)
    }

    fn align_items(self, align_items: AlignItems) -> Chain<Self, AlignItems> {
        self.chain(align_items)
    }

    fn flex_basis(self, dimension: Dimension) -> Chain<Self, FlexBasis> {
        self.chain(FlexBasis { dimension })
    }

    fn flex_direction(self, flex_direction: FlexDirection) -> Chain<Self, FlexDirection> {
        self.chain(flex_direction)
    }

    fn flex_grow(self, value: f32) -> Chain<Self, FlexGrow> {
        self.chain(FlexGrow { value })
    }

    fn flex_shrink(self, value: f32) -> Chain<Self, FlexShrink> {
        self.chain(FlexShrink { value })
    }

    fn gap(self, gap: Gap) -> Chain<Self, Gap> {
        self.chain(gap)
    }

    fn justify_content(self, justify_content: JustifyContent) -> Chain<Self, JustifyContent> {
        self.chain(justify_content)
    }

    fn keyboard_handler<H>(self, handler: H) -> Chain<Self, KeyboardInput<H>>
    where
        H: KeyboardHandler + 'static,
    {
        self.chain(KeyboardInput::new(handler))
    }

    fn margin(self, rect: Rect<Dimension>) -> Chain<Self, Margin> {
        self.chain(Margin { rect })
    }

    fn padding(self, padding: Padding) -> Chain<Self, Padding> {
        self.chain(padding)
    }

    fn role(self, role: Role) -> Chain<Self, Role> {
        self.chain(role)
    }
}

impl<M: Modify<ContainerConfig>> ContainerModifier for M {}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            merge_descendants: false,
            role: Role::default(),
            style: Style::default(),
        }
    }
}

impl AsMut<Role> for ContainerConfig {
    fn as_mut(&mut self) -> &mut Role {
        &mut self.role
    }
}

impl AsMut<Style> for ContainerConfig {
    fn as_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl AsMut<Size<Dimension>> for ContainerConfig {
    fn as_mut(&mut self) -> &mut Size<Dimension> {
        &mut self.style.size
    }
}

pub struct MergeDescendants;

impl Modify<ContainerConfig> for MergeDescendants {
    fn modify(&mut self, value: &mut ContainerConfig) {
        value.merge_descendants = true;
    }
}

impl<T: AsMut<Style>> Modify<T> for AlignItems {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().align_items = *self;
    }
}

impl<T: AsMut<Style>> Modify<T> for JustifyContent {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().justify_content = *self;
    }
}

impl<T: AsMut<Style>> Modify<T> for FlexDirection {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().flex_direction = *self;
    }
}


#[derive(Default)]
pub struct Gap {
    size: Size<Dimension>,
}

impl Gap {
    pub fn width(mut self, value: Dimension) -> Self {
        self.size.width = value;
        self
    }

    pub fn height(mut self, value: Dimension) -> Self {
        self.size.height = value;
        self
    }
}

impl<T: AsMut<Style>> Modify<T> for Gap {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().gap = self.size;
    }
}

#[derive(Default)]
pub struct Padding {
    rect: Rect<Dimension>,
}

impl Padding {
    pub fn left(mut self, value: Dimension) -> Self {
        self.rect.left = value;
        self
    }

    pub fn right(mut self, value: Dimension) -> Self {
        self.rect.right = value;
        self
    }

    pub fn horizontal(self, value: Dimension) -> Self {
        self.left(value).right(value)
    }

    pub fn top(mut self, value: Dimension) -> Self {
        self.rect.top = value;
        self
    }

    pub fn bottom(mut self, value: Dimension) -> Self {
        self.rect.bottom = value;
        self
    }

    pub fn vertical(self, value: Dimension) -> Self {
        self.top(value).bottom(value)
    }
}

impl From<Dimension> for Padding {
    fn from(value: Dimension) -> Self {
        Self::default().horizontal(value).vertical(value)
    }
}

impl<T: AsMut<Style>> Modify<T> for Padding {
    fn modify(&mut self, value: &mut T) {
        value.as_mut().padding = self.rect;
    }
}
