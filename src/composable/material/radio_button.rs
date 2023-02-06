use accesskit::Role;
use skia_safe::{Color4f, Paint, PaintStyle};
use taffy::{
    geometry::Point,
    prelude::{Layout, Size},
};

use crate::{
    composable::container,
    modify::{Chain, HandlerModifier, ModifyExt},
    CanvasExt, DevicePixels, Modifier, Modify,
};

#[derive(Default)]
pub struct RadioButtonConfig {
    on_click: Option<Box<dyn FnMut()>>,
}

pub struct OnClick {
    f: Option<Box<dyn FnMut()>>,
}

impl Modify<RadioButtonConfig> for OnClick {
    fn modify(&mut self, value: &mut RadioButtonConfig) {
        value.on_click = self.f.take();
    }
}

pub trait RadioButtonModifier: Modify<RadioButtonConfig> + Sized {
    fn on_click(self, on_click: impl FnMut() + 'static) -> Chain<RadioButtonConfig, Self, OnClick> {
        self.chain(OnClick {
            f: Some(Box::new(on_click)),
        })
    }
}

impl<M> RadioButtonModifier for M where M: Modify<RadioButtonConfig> {}

#[track_caller]
pub fn radio_button(mut modifier: impl Modify<RadioButtonConfig>) {
    let outer_radius = 20.dp();
    let inner_radius = 12.dp();
    let stroke_width = 2.dp();

    let mut config = RadioButtonConfig::default();
    modifier.modify(&mut config);

    let clickable = config
        .on_click
        .map(|on_click| Modifier.clickable(Role::RadioButton, on_click));

    container(
        clickable
            .size(Size::from_points(outer_radius, outer_radius))
            .draw(move |layout, canvas| {
                let mut paint = Paint::new(Color4f::new(255., 0., 0., 1.), None);
                paint.set_stroke(true);
                paint.set_stroke_width(stroke_width);
                paint.set_style(PaintStyle::Stroke);
                canvas.circle(layout, &paint);

                let edge_size = (outer_radius - inner_radius) / 2.;
                let inner_layout = Layout {
                    order: layout.order,
                    size: Size {
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
            }),
        || {},
    )
}
