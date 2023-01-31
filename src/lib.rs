use std::any::Any;

pub mod composer;
pub use composer::Composer;

mod semantics;
pub use semantics::Semantics;

mod text;
pub use text::text;

pub trait Widget: Any {
    fn semantics(&mut self, semantics: &mut Semantics);
}
