use super::{Element, Native};
use crate::View;

use slotmap::DefaultKey;
use taffy::{
    prelude::{Layout, Size},
    style::Style,
    Taffy,
};

pub fn canvas<F>(draw: F) -> Canvas<F>
where
    F: FnMut(&Layout, &mut skia_safe::Canvas) + 'static,
{
    Canvas { draw }
}

pub struct Canvas<F> {
    draw: F,
}

impl<E, F> View<Native<E>> for Canvas<F>
where
    F: FnMut(&Layout, &mut skia_safe::Canvas) + 'static,
{
    type State = ();

    fn build(self, cx: &mut Native<E>) -> Self::State {
        let mut layout = Style::default();
        layout.size = Size::from_points(200., 200.);
        let layout_key = cx.taffy.new_leaf(layout).unwrap();
        cx.layout_stack.push(layout_key);

        let key = cx.elements.insert(Box::new(CanvasElement {
            layout_key,
            draw: self.draw,
        }));
        cx.stack.push(key);
    }

    fn rebuild(self, _cx: &mut Native<E>, _state: &mut Self::State) {}

    fn remove(_cx: &mut Native<E>, _state: &mut Self::State) {}
}

struct CanvasElement<F> {
    layout_key: DefaultKey,
    draw: F,
}

impl<F> Element for CanvasElement<F>
where
    F: FnMut(&Layout, &mut skia_safe::Canvas),
{
    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas) {
        let layout = taffy.layout(self.layout_key).unwrap();
        (self.draw)(layout, canvas)
    }
}
