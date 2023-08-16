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

impl<T, A, V1, V2> View<T, A> for (V1, V2)
where
    V1: View<T, A>,
    V2: View<T, A>,
{
    type State = (V1::State, V2::State);

    fn view(&mut self, _state: &mut T, _id_path: &[Id], _message: Box<dyn Any>) {}

    fn layout(&mut self, cx: &mut LayoutContext) {
        self.0.layout(cx);
        self.1.layout(cx);
    }

    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas) {
        self.0.paint(taffy, canvas);
        self.1.paint(taffy, canvas);
    }
}
