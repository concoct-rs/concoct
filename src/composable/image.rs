use super::Container;
use crate::{
    dimension::{DevicePixels, Size},
    modify::ModifyExt,
    Modifier, View,
};
use skia_safe::Data;
use taffy::style::Dimension;

#[must_use]
pub struct Image {
    data: Data,
}

impl Image {
    pub fn build(data: impl AsRef<[u8]>) -> Self {
        Self {
            data: Data::new_copy(data.as_ref()),
        }
    }

    #[track_caller]
    pub fn new(data: impl AsRef<[u8]>) {
        Self::build(data).view()
    }
}

impl View for Image {
    #[track_caller]
    fn view(self) {
        Container::build_row(|| {})
            .size(Size::from(Dimension::Points(200.dp())))
            .modifier(Modifier.draw(move |layout, canvas| {
                let image = skia_safe::Image::from_encoded(self.data.clone()).unwrap();
                canvas.draw_image(image, (layout.location.x, layout.location.y), None);
            }))
            .view();
    }
}
