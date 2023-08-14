use super::ElementKey;
use crate::{LayoutContext, UserEvent};
use taffy::Taffy;

mod canvas;
pub use canvas::Canvas;

mod group;
pub use group::Group;
use winit::event_loop::EventLoopProxy;

pub trait Element {
    fn spawn(&mut self, _key: ElementKey, _proxy: EventLoopProxy<UserEvent>) {}

    fn layout(&mut self, key: ElementKey, cx: LayoutContext);

    fn semantics(&mut self, taffy: &Taffy);

    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas);

    fn children(&mut self, children: &mut Vec<ElementKey>);
}
