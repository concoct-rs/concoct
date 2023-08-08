use super::Element;
use crate::{ElementKey, LayoutContext};
use skia_safe::Rect;
use slotmap::DefaultKey;
use taffy::{prelude::Layout, style::Style, Taffy};

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

impl<F> Element for Canvas<F>
where
    F: FnMut(&Layout, &mut skia_safe::Canvas),
{
    fn layout(&mut self, key: ElementKey, cx: LayoutContext) {
        let layout_key = cx.insert(key, self.style.clone());
        self.layout_key = Some(layout_key);
    }

    fn semantics(&mut self, _taffy: &Taffy) {
        todo!()
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

    fn children(&mut self, _children: &mut Vec<ElementKey>) {}
}
