use super::Element;
use crate::render::{ElementKey, LayoutContext};
use skia_safe::Rect;
use slotmap::DefaultKey;
use taffy::{style::Style, Taffy};

pub struct Canvas {
    layout_key: Option<DefaultKey>,
    draw: Box<dyn FnMut(&Taffy, &mut skia_safe::Canvas)>,
    pub style: Style,
}

impl Canvas {
    pub fn new(draw: Box<dyn FnMut(&Taffy, &mut skia_safe::Canvas)>) -> Self {
        Self {
            draw,
            layout_key: None,
            style: Style::default(),
        }
    }
}

impl Element for Canvas {
    fn layout(&mut self, key: ElementKey, cx: LayoutContext) -> bool {
        let layout_key = cx.taffy.new_leaf(self.style.clone()).unwrap();

        cx.layout_elements.insert(layout_key, key);
        cx.element_layouts.insert(key, layout_key);

        self.layout_key = Some(layout_key);
        true
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

        (self.draw)(taffy, canvas);

        canvas.restore();
    }

    fn children(&mut self, _children: &mut Vec<ElementKey>) {}
}
