use std::collections::HashMap;

use accesskit::Point;
use skia_safe::Canvas;
use slotmap::{new_key_type, DefaultKey, SlotMap};
use taffy::{
    compute_layout,
    prelude::{Layout, Size},
    style::{AvailableSpace, Style},
    Taffy,
};

use super::element::Element;

pub struct LayoutContext<'a> {
    pub taffy: &'a mut Taffy,
    pub layout_elements: &'a mut HashMap<DefaultKey, ElementKey>,
    pub element_layouts: &'a mut HashMap<ElementKey, DefaultKey>,
}

impl<'a> LayoutContext<'a> {
    pub fn insert(self, key: ElementKey, style: Style) -> DefaultKey {
        let layout_key = self.taffy.new_leaf(style).unwrap();
        self.layout_elements.insert(layout_key, key);
        self.element_layouts.insert(key, layout_key);
        layout_key
    }

    pub fn insert_with_children(
        self,
        key: ElementKey,
        style: Style,
        children: &[ElementKey],
    ) -> DefaultKey {
        let layout_children: Vec<_> = children
            .iter()
            .filter_map(|child| self.element_layouts.get(child))
            .cloned()
            .collect();
        let layout_key = self
            .taffy
            .new_with_children(style, &layout_children)
            .unwrap();

        self.layout_elements.insert(layout_key, key);
        self.element_layouts.insert(key, layout_key);

        layout_key
    }
}

new_key_type! {
    pub struct ElementKey;
}

#[derive(Default)]
pub struct Tree {
    taffy: Taffy,
    elements: SlotMap<ElementKey, Box<dyn Element>>,
    layout_elements: HashMap<DefaultKey, ElementKey>,
    element_layouts: HashMap<ElementKey, DefaultKey>,
}

impl Tree {
    pub fn insert(&mut self, element: Box<dyn Element>) -> ElementKey {
        let key = self.elements.insert(element);
        self.elements.get_mut(key).unwrap().layout(
            key,
            LayoutContext {
                taffy: &mut self.taffy,
                layout_elements: &mut self.layout_elements,
                element_layouts: &mut self.element_layouts,
            },
        );
        key
    }

    pub fn remove(&mut self, key: ElementKey) -> Option<Box<dyn Element>> {
        self.elements.remove(key)
    }

    pub fn get_mut(&mut self, key: ElementKey) -> Option<&mut Box<dyn Element>> {
        self.elements.get_mut(key)
    }

    pub fn visit(&mut self, root: ElementKey, visitor: impl FnMut(&mut Box<dyn Element>)) {
        visit(&mut self.elements, root, visitor)
    }

    pub fn visit_layout(&self, root: ElementKey, mut visit: impl FnMut(DefaultKey, &Layout)) {
        let layout_key = *self.element_layouts.get(&root).unwrap();
        let mut keys = vec![layout_key];

        while let Some(key) = keys.pop() {
            let layout = self.taffy.layout(key).unwrap();
            visit(key, layout);

            let children = self.taffy.children(key).unwrap();
            keys.extend_from_slice(&children);
        }
    }

    pub fn target(&self, root: ElementKey, point: Point) -> Option<ElementKey> {
        let mut target: Option<(DefaultKey, Layout)> = None;

        self.visit_layout(root, |key, layout| {
            if point.x >= layout.location.x as _
                && point.x <= (layout.location.x + layout.size.width) as _
                && point.y >= layout.location.y as _
                && point.y <= (layout.location.y + layout.size.height) as _
            {
                if let Some((_, target_layout)) = target {
                    if layout.order >= target_layout.order {
                        target = Some((key, layout.clone()));
                    }
                } else {
                    target = Some((key, layout.clone()));
                }
            }
        });

        target.map(|(key, _)| self.layout_elements.get(&key).unwrap().clone())
    }

    pub fn paint(&mut self, root: ElementKey, canvas: &mut Canvas) {
        visit(&mut self.elements, root, |elem| {
            elem.paint(&self.taffy, canvas)
        });
    }

    pub fn layout(&mut self, root: ElementKey, available_space: Size<AvailableSpace>) {
        let layout_key = *self.element_layouts.get(&root).unwrap();
        compute_layout(&mut self.taffy, layout_key, available_space).unwrap();
    }
}

fn visit(
    elements: &mut SlotMap<ElementKey, Box<dyn Element>>,
    root: ElementKey,
    mut visit: impl FnMut(&mut Box<dyn Element>),
) {
    let mut keys = vec![root];

    while let Some(key) = keys.pop() {
        let elem = elements.get_mut(key).unwrap();
        visit(elem);
        elem.children(&mut keys);
    }
}
