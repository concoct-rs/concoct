use crate::Id;
use std::any::Any;

mod adapt;
pub use adapt::{Adapt, AdaptThunk};

mod canvas;
pub use canvas::Canvas;

mod text;
use taffy::{prelude::Node, style::Style, Taffy};
pub use text::Text;

pub struct LayoutContext {
    pub taffy: Taffy,
    pub children: Vec<Node>,
}

impl LayoutContext {
    pub fn push(&mut self, style: Style) -> Node {
        let key = self.taffy.new_leaf(style).unwrap();
        self.children.push(key);
        key
    }
}

pub trait View<T, A> {
    type State;

    fn view(&mut self, state: &mut T, id_path: &[Id], message: Box<dyn Any>);

    fn layout(&mut self, _cx: &mut LayoutContext) {}

    fn paint(&mut self, _taffy: &Taffy, _canvas: &mut skia_safe::Canvas) {}
}
