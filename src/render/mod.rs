use std::collections::HashMap;

use accesskit::Point;
use element::Element;
use skia_safe::Canvas;
use slotmap::{DefaultKey, SlotMap};
use taffy::{
    compute_layout,
    prelude::{Layout, Size},
    style::AvailableSpace,
    Taffy,
};

pub mod renderer;

pub mod element;

pub struct LayoutContext<'a> {
    taffy: &'a mut Taffy,
    layout_elements: &'a mut HashMap<DefaultKey, DefaultKey>,
    element_layouts: &'a mut HashMap<DefaultKey, DefaultKey>,
}

#[derive(Default)]
pub struct Tree {
    taffy: Taffy,
    elements: SlotMap<DefaultKey, Box<dyn Element>>,
    layout_elements: HashMap<DefaultKey, DefaultKey>,
    element_layouts: HashMap<DefaultKey, DefaultKey>,
}

impl Tree {
    pub fn insert(&mut self, element: Box<dyn Element>) -> DefaultKey {
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

    pub fn get_mut(&mut self, key: DefaultKey) -> Option<&mut Box<dyn Element>> {
        self.elements.get_mut(key)
    }

    pub fn visit(&mut self, root: DefaultKey, visitor: impl FnMut(&mut Box<dyn Element>)) {
        visit(&mut self.elements, root, visitor)
    }

    pub fn visit_layout(&self, root: DefaultKey, mut visit: impl FnMut(DefaultKey, &Layout)) {
        let layout_key = *self.layout_elements.get(&root).unwrap();
        let mut keys = vec![layout_key];

        while let Some(key) = keys.pop() {
            let layout = self.taffy.layout(key).unwrap();
            visit(key, layout);

            let children = self.taffy.children(key).unwrap();
            keys.extend_from_slice(&children);
        }
    }

    pub fn target(&self, root: DefaultKey, point: Point) -> Option<DefaultKey> {
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

    pub fn paint(&mut self, root: DefaultKey, canvas: &mut Canvas) {
        visit(&mut self.elements, root, |elem| {
            elem.paint(&self.taffy, canvas)
        });
    }

    pub fn layout(&mut self, root: DefaultKey, available_space: Size<AvailableSpace>) {
        compute_layout(&mut self.taffy, root, available_space).unwrap();
    }
}

fn visit(
    elements: &mut SlotMap<DefaultKey, Box<dyn Element>>,
    root: DefaultKey,
    mut visit: impl FnMut(&mut Box<dyn Element>),
) {
    let mut keys = vec![root];

    while let Some(key) = keys.pop() {
        let elem = elements.get_mut(key).unwrap();
        visit(elem);
        elem.children(&mut keys);
    }
}
