use std::any::Any;

pub mod composer;
pub use composer::Composer;

mod container;
pub use container::container;

pub mod modify;
pub use modify::{Modifier, Modify};

mod semantics;
pub use semantics::Semantics;

mod text;
pub use text::text;

pub trait Widget: Any {
    fn semantics(&mut self, semantics: &mut Semantics);

    fn any_mut(&mut self) -> &mut dyn Any;
}
