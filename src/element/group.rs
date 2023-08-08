use super::Element;
use crate::{ElementKey, LayoutContext};
use skia_safe::Canvas;
use slotmap::DefaultKey;
use taffy::{style::Style, Taffy};

pub struct Group {
    layout_key: Option<DefaultKey>,
    pub style: Style,
    pub children: Vec<ElementKey>,
}

impl Group {
    pub fn new(style: Style, children: Vec<ElementKey>) -> Self {
        Self {
            layout_key: None,
            style,
            children,
        }
    }
}

impl Element for Group {
    fn layout(&mut self, key: ElementKey, cx: LayoutContext) {
        let layout_key = cx.insert_with_children(key, self.style.clone(), &self.children);
        self.layout_key = Some(layout_key);
    }

    fn semantics(&mut self, _taffy: &Taffy) {
        todo!()
    }

    fn paint(&mut self, _taffy: &Taffy, _canvas: &mut Canvas) {}

    fn children(&mut self, children: &mut Vec<ElementKey>) {
        children.extend_from_slice(&self.children);
    }
}
