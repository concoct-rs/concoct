use super::Element;
use crate::render::{ElementKey, LayoutContext};
use skia_safe::Rect;
use slotmap::DefaultKey;
use taffy::{prelude::Layout, style::Style, Taffy};

pub struct Canvas {
    layout_key: Option<DefaultKey>,
    draw: Box<dyn FnMut(&Layout, &mut skia_safe::Canvas)>,
    pub style: Style,
}

impl Canvas {
    pub fn new(draw: Box<dyn FnMut(&Layout, &mut skia_safe::Canvas)>) -> Self {
        Self {
            draw,
            layout_key: None,
            style: Style::default(),
        }
    }
}

impl Element for Canvas {
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
