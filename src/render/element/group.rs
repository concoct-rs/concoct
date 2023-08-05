use super::Element;
use crate::render::{ElementKey, LayoutContext};
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
    fn layout(&mut self, key: ElementKey, cx: LayoutContext) -> bool {
        let layout_children: Vec<_> = self
            .children
            .iter()
            .map(|child| *cx.element_layouts.get(child).unwrap())
            .collect();
        let layout_key = cx
            .taffy
            .new_with_children(self.style.clone(), &layout_children)
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

    fn children(&mut self, children: &mut Vec<ElementKey>) {
        children.extend_from_slice(&self.children);
    }
}
