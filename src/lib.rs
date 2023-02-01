use std::any::Any;

pub mod composer;
pub use composer::Composer;

pub mod composable;

mod container;
pub use container::container;

pub mod modify;
pub use modify::{Modifier, Modify};

pub mod render;

pub mod semantics;
pub use semantics::Semantics;

pub mod state;

mod tester;
use skia_safe::Canvas;
pub use tester::Tester;

mod text;
pub use text::text;

pub trait Widget: Any {
    fn semantics(&mut self, semantics: &mut Semantics);

    fn paint(&mut self, semantics: &Semantics, canvas: &mut Canvas);

    fn remove(&mut self, semantics: &mut Semantics);

    fn any(&self) -> &dyn Any;

    fn any_mut(&mut self) -> &mut dyn Any;
}
