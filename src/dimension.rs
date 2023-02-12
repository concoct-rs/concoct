use crate::Composer;
use taffy::style::Dimension;

pub trait DevicePixels {
    fn dp(self) -> f32;
}

impl DevicePixels for f32 {
    fn dp(self) -> f32 {
        Composer::with(|composer| self * composer.borrow().scale_factor)
    }
}

impl DevicePixels for i32 {
    fn dp(self) -> f32 {
        (self as f32).dp()
    }
}

#[derive(Default)]
pub struct Size {
    inner: taffy::prelude::Size<Dimension>,
}

impl Size {
    pub fn width(mut self, value: Dimension) -> Self {
        self.inner.width = value;
        self
    }

    pub fn height(mut self, value: Dimension) -> Self {
        self.inner.height = value;
        self
    }
}

impl From<taffy::prelude::Size<Dimension>> for Size {
    fn from(value: taffy::prelude::Size<Dimension>) -> Self {
        Self { inner: value }
    }
}

impl From<Dimension> for Size {
    fn from(value: Dimension) -> Self {
        Self::from(taffy::prelude::Size {
            width: value,
            height: value,
        })
    }
}

impl From<Size> for taffy::prelude::Size<Dimension> {
    fn from(value: Size) -> Self {
        value.inner
    }
}

#[derive(Clone, Copy, Default)]
pub struct Padding {
    pub rect: taffy::prelude::Rect<Dimension>,
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
