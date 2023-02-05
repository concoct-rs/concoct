use std::any::Any;

pub mod composer;
use accesskit::Action;
pub use composer::Composer;

pub mod composable;

pub mod modify;
pub use modify::{Modifier, Modify};

pub mod render;

pub mod semantics;
pub use semantics::Semantics;

mod tester;
use skia_safe::Canvas;
use taffy::style::Dimension;
pub use tester::Tester;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Touch, VirtualKeyCode},
};

pub trait Widget: Any {
    fn layout(&mut self, semantics: &mut Semantics);

    fn semantics(&mut self, semantics: &mut Semantics);

    fn paint(&mut self, semantics: &Semantics, canvas: &mut Canvas);

    fn remove(&mut self, semantics: &mut Semantics);

    fn any(&self) -> &dyn Any;

    fn any_mut(&mut self) -> &mut dyn Any;
}

pub enum Event {
    Action(Action),
    KeyboardInput {
        state: ElementState,
        key_code: VirtualKeyCode,
    },
    MouseInput {
        state: ElementState,
        cursor: PhysicalPosition<f64>,
    },
    Touch(Touch),
}

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
