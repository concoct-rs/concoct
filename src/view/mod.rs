use std::{any::Any, num::NonZeroU128};
use taffy::{prelude::Node, style::Style, Taffy};

mod adapt;
pub use adapt::{Adapt, AdaptThunk};

mod canvas;
pub use canvas::Canvas;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id(NonZeroU128);

pub struct BuildContext {
    pub next_id: NonZeroU128,
    pub unused_ids: Vec<Id>,
}

impl BuildContext {
    pub fn id(&mut self) -> Id {
        self.unused_ids.pop().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id = self.next_id.checked_add(1).unwrap();
            Id(id)
        })
    }
}

pub trait View<T, A> {
    type State;

    fn build(&mut self, cx: &mut BuildContext) -> (Id, Self::State);

    fn rebuild(&mut self, cx: &mut BuildContext, old: &mut Self) {}

    fn layout(&mut self, _cx: &mut LayoutContext);

    fn paint(&mut self, _taffy: &Taffy, _canvas: &mut skia_safe::Canvas);

    fn message(&mut self, state: &mut T, id_path: &[Id], message: &dyn Any);
}

impl<T, A, V1, V2> View<T, A> for (V1, V2)
where
    V1: View<T, A>,
    V2: View<T, A>,
{
    type State = (V1::State, V2::State);

    fn build(&mut self, cx: &mut BuildContext) -> (Id, Self::State) {
        let (_a, b) = self.0.build(cx);
        let (_c, d) = self.1.build(cx);

        let id = cx.id();
        (id, (b, d))
    }

    fn layout(&mut self, cx: &mut LayoutContext) {
        self.0.layout(cx);
        self.1.layout(cx);
    }

    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas) {
        self.0.paint(taffy, canvas);
        self.1.paint(taffy, canvas);
    }

    fn message(&mut self, state: &mut T, id_path: &[Id], message: &dyn Any) {
        self.0.message(state, id_path, message);
        self.1.message(state, id_path, message);
    }
}
