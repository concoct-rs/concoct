use super::ElementKey;
use crate::LayoutContext;
use taffy::Taffy;

mod canvas;
pub use canvas::Canvas;

mod group;
pub use group::Group;

pub trait Element {
    fn layout(&mut self, key: ElementKey, cx: LayoutContext);

    fn semantics(&mut self, taffy: &Taffy);

    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas);

    fn children(&mut self, children: &mut Vec<ElementKey>);
}
