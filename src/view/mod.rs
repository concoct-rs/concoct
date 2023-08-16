use accesskit::Point;
use std::{any::Any, num::NonZeroU128};
use taffy::{
    prelude::{Layout, Node},
    style::Style,
    Taffy,
};

mod adapt;
pub use adapt::{Adapt, AdaptThunk};

mod canvas;
pub use canvas::Canvas;

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

pub struct LayoutContext {
    pub taffy: Taffy,
    pub children: Vec<Node>,
    pub root: Node,
}

impl LayoutContext {
    pub fn push(&mut self, style: Style) -> Node {
        let key = self.taffy.new_leaf(style).unwrap();
        self.children.push(key);
        key
    }

    pub fn iter(&self) -> Iter {
        Iter {
            taffy: &self.taffy,
            keys: vec![self.root],
        }
    }

    pub fn targets(&self, point: Point) -> impl Iterator<Item = (Node, &Layout)> {
        self.iter().filter(move |(_key, layout)| {
            layout.location.x <= point.x as _
                && layout.location.x + layout.size.width >= point.x as _
                && layout.location.y <= point.y as _
                && layout.location.y + layout.size.height >= point.y as _
        })
    }
}

pub struct Iter<'a> {
    taffy: &'a Taffy,
    keys: Vec<Node>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Node, &'a Layout);

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.pop().map(|key| {
            let children = self.taffy.children(key).unwrap();
            self.keys.extend_from_slice(&children);

            let layout = self.taffy.layout(key).unwrap();
            (key, layout)
        })
    }
}

pub trait View<T, A = ()> {
    type State;

    fn build(&mut self, cx: &mut BuildContext) -> (Id, Self::State);

    fn rebuild(&mut self, cx: &mut BuildContext, old: &mut Self);

    fn layout(&mut self, cx: &mut LayoutContext);

    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas);

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

    fn rebuild(&mut self, cx: &mut BuildContext, old: &mut Self) {
        self.0.rebuild(cx, &mut old.0);
        self.1.rebuild(cx, &mut old.1);
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
