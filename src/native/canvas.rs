use super::{Element, Native};
use crate::View;
use skia_safe::{Color4f, Paint};
use slotmap::DefaultKey;
use taffy::{prelude::Size, style::Style, Taffy};

pub struct Canvas {}

impl<E> View<Native<E>> for Canvas {
    type State = ();

    fn build(self, cx: &mut Native<E>) -> Self::State {
        let mut layout = Style::default();
        layout.size = Size::from_points(200., 200.);
        let layout_key = cx.taffy.new_leaf(layout).unwrap();
        cx.layout_stack.push(layout_key);

        let key = cx.elements.insert(Box::new(CanvasElement { layout_key }));
        cx.stack.push(key);
    }

    fn rebuild(self, _cx: &mut Native<E>, _state: &mut Self::State) {}

    fn remove(_cx: &mut Native<E>, _state: &mut Self::State) {}
}

struct CanvasElement {
    layout_key: DefaultKey,
}

impl Element for CanvasElement {
    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas) {
        let layout = taffy.layout(self.layout_key).unwrap();
        canvas.draw_circle(
            (layout.location.x, layout.location.y),
            100.,
            &Paint::new(Color4f::new(1., 0., 0., 1.), None),
        );
    }
}
