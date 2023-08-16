use super::Id;
use accesskit::Point;
use std::collections::HashMap;
use taffy::{
    prelude::{Layout, Node, Size},
    style::Style,
    style_helpers::TaffyMaxContent,
    Taffy,
};

pub struct LayoutContext {
    pub taffy: Taffy,
    pub children: Vec<Node>,
    pub root: Node,
    pub keys: HashMap<Node, Id>,
}

impl Default for LayoutContext {
    fn default() -> Self {
        let mut taffy = Taffy::new();
        let root = taffy.new_leaf(Style::DEFAULT).unwrap();

        Self {
            taffy,
            children: Vec::new(),
            root,
            keys: HashMap::new(),
        }
    }
}

impl LayoutContext {
    pub fn insert(&mut self, id: Id, style: Style) -> Node {
        let key = self.taffy.new_leaf(style).unwrap();
        self.children.push(key);
        self.keys.insert(key, id);
        key
    }

    pub fn iter(&self) -> Iter {
        Iter {
            taffy: &self.taffy,
            keys: vec![self.root],
        }
    }

    pub fn targets(&self, point: Point) -> impl Iterator<Item = Id> + '_ {
        self.iter().filter_map(move |(key, layout)| {
            if layout.location.x <= point.x as _
                && layout.location.x + layout.size.width >= point.x as _
                && layout.location.y <= point.y as _
                && layout.location.y + layout.size.height >= point.y as _
            {
                self.keys.get(&key).copied()
            } else {
                None
            }
        })
    }

    pub fn compute_layout(&mut self) {
        self.taffy.set_children(self.root, &self.children).unwrap();

        taffy::compute_layout(&mut self.taffy, self.root, Size::MAX_CONTENT).unwrap();
    }
}

pub struct Iter<'a> {
    taffy: &'a Taffy,
    keys: Vec<Node>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Node, &'a Layout);

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.pop().map(|key| {
            let children = self.taffy.children(key).unwrap();
            self.keys.extend_from_slice(&children);

            let layout = self.taffy.layout(key).unwrap();
            (key, layout)
        })
    }
}
