use super::{Element, Native};
use crate::View;
use skia_safe::{Color4f, Paint};

pub struct Canvas {}

impl<E> View<Native<E>> for Canvas {
    type State = ();

    fn build(self, cx: &mut Native<E>) -> Self::State {
        let key = cx.elements.insert(Box::new(CanvasElement {}));
        cx.stack.push(key);
    }

    fn rebuild(self, _cx: &mut Native<E>, _state: &mut Self::State) {}

    fn remove(_cx: &mut Native<E>, _state: &mut Self::State) {}
}

struct CanvasElement {}
impl Element for CanvasElement {
    fn paint(&mut self, canvas: &mut skia_safe::Canvas) {
        canvas.draw_circle(
            (100., 100.),
            100.,
            &Paint::new(Color4f::new(1., 0., 0., 1.), None),
        );
    }
}
