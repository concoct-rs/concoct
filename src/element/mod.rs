mod dom;
pub use dom::DomElement;

mod text;
pub use text::TextElement;

use crate::ElementContext;

pub trait Element {
    type State;

    fn build(&self, cx: &mut ElementContext) -> Self::State;
}
