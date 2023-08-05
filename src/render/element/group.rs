use super::Element;
use crate::render::LayoutContext;
use skia_safe::Canvas;
use slotmap::DefaultKey;
use taffy::{style::Style, Taffy};

pub struct Group {
    layout_key: Option<DefaultKey>,
    pub style: Style,
    pub children: Vec<DefaultKey>,
}

impl Group {
    pub fn new(style: Style, children: Vec<DefaultKey>) -> Self {
        Self {
            layout_key: None,
            style,
            children,
        }
    }
}

impl Element for Group {
    fn layout(&mut self, key: DefaultKey, cx: LayoutContext) -> bool {
        let layout_key = cx
            .taffy
            .new_with_children(self.style.clone(), &self.children)
            .unwrap();

        cx.layout_elements.insert(layout_key, key);
        cx.element_layouts.insert(key, layout_key);

        self.layout_key = Some(layout_key);
        true
    }

    fn semantics(&mut self, _taffy: &Taffy) {
        todo!()
    }

    fn paint(&mut self, _taffy: &Taffy, _canvas: &mut Canvas) {}

    fn children(&mut self, children: &mut Vec<DefaultKey>) {
        children.extend_from_slice(&self.children);
    }
}
