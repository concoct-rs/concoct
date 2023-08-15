use skia_safe::Rect;
use slotmap::DefaultKey;
use taffy::{prelude::Layout, style::Style, Taffy};

use super::{LayoutContext, View};

/// Canvas element.
/// This lets you draw directly to the skia canvas.
pub struct Canvas<F> {
    layout_key: Option<DefaultKey>,
    pub draw: F,
    pub style: Style,
}

impl<F> Canvas<F>
where
    F: FnMut(&Layout, &mut skia_safe::Canvas),
{
    /// Create a new canvas element that will draw its content with the given function.
    pub fn new(draw: F) -> Self {
        Self {
            draw,
            layout_key: None,
            style: Style::default(),
        }
    }
}

impl<T, A, F> View<T, A> for Canvas<F>
where
    F: FnMut(&Layout, &mut skia_safe::Canvas),
{
    type State = ();

    fn view(&mut self, _state: &mut T, _id_path: &[crate::Id], _message: Box<dyn std::any::Any>) {}

    fn layout(&mut self, cx: &mut LayoutContext) {
        if self.layout_key.is_none() {
            let layout_key = cx.push(self.style.clone());
            self.layout_key = Some(layout_key);
        }
    }

    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas) {
        let layout = taffy.layout(self.layout_key.unwrap()).unwrap();
        canvas.save();
        canvas.clip_rect(
            Rect::new(
                layout.location.x,
                layout.location.y,
                layout.location.x + layout.size.width,
                layout.location.y + layout.size.height,
            ),
            None,
            None,
        );
        canvas.translate((layout.location.x, layout.location.y));

        (self.draw)(layout, canvas);

        canvas.restore();
    }
}
