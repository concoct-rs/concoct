use crate::{
    composable::container,
    dimension::{DevicePixels, Size},
    modify::{HandlerModifier, ModifyExt},
    CanvasExt, Modifier, View,
};
use accesskit::Role;
use container::Container;
use skia_safe::{Color4f, Paint, PaintStyle};
use taffy::{geometry::Point, prelude::Layout, style::Dimension};

#[must_use]
pub struct RadioButton {
    on_click: Option<Box<dyn FnMut()>>,
}

impl RadioButton {
    pub fn build() -> Self {
        Self { on_click: None }
    }

    pub fn new() {
        Self::build().view()
    }

    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl View for RadioButton {
    #[track_caller]
    fn view(self) {
        let outer_radius = 20.dp();
        let inner_radius = 12.dp();
        let stroke_width = 2.dp();

        let clickable = self
            .on_click
            .map(|on_click| Modifier.clickable(Role::RadioButton, on_click));

        Container::build(|| {}, Role::RadioButton)
            .size(Size::from(Dimension::Points(outer_radius)))
            .modifier(clickable.draw(move |layout, canvas| {
                let mut paint = Paint::new(Color4f::new(255., 0., 0., 1.), None);
                paint.set_stroke(true);
                paint.set_stroke_width(stroke_width);
                paint.set_style(PaintStyle::Stroke);
                canvas.circle(layout, &paint);

                let edge_size = (outer_radius - inner_radius) / 2.;
                let inner_layout = Layout {
                    order: layout.order,
                    size: taffy::prelude::Size {
                        width: inner_radius,
                        height: inner_radius,
                    },
                    location: Point {
                        x: layout.location.x + edge_size,
                        y: layout.location.y + edge_size,
                    },
                };

                let paint = Paint::new(Color4f::new(0., 255., 0., 1.), None);
                canvas.circle(&inner_layout, &paint);
            }))
            .view()
    }
}
