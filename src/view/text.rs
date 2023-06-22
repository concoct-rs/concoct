use crate::{Context, SemanticsContext, View, Widget};
use accesskit::{NodeBuilder, NodeClassSet, NodeId, Role};
use skia_safe::{
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    Color4f, FontMgr, Typeface, RGB,
};
use std::sync::{Arc, Mutex};
use taffy::{
    prelude::Size,
    style::{AvailableSpace, Style},
};

pub struct Text {
    pub string: String,
    pub color: Color4f,
    pub typeface: Typeface,
    pub style: Style,
    pub font_size: f32,
}

impl Text {
    pub fn new(string: impl Into<String>) -> Self {
        Self {
            string: string.into(),
            color: Color4f::new(0., 0., 0., 1.),
            typeface: Typeface::new("serif", Default::default()).unwrap(),
            style: Style::default(),
            font_size: 24.,
        }
    }
}

impl View for Text {
    fn view(&mut self, cx: &mut Context) {
        let font_mgr = FontMgr::new();
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(font_mgr, None);

        let paragraph_style = ParagraphStyle::new();
        let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);

        let mut text_style = TextStyle::new();
        text_style.set_color(RGB::from((0, 0, 0)));
        text_style.set_font_size(self.font_size);
        text_style.set_typeface(self.typeface.clone());
        paragraph_builder.push_style(&text_style);

        paragraph_builder.add_text(&self.string);
        paragraph_builder.pop();

        let paragraph = Arc::new(Mutex::new(paragraph_builder.build()));

        let widget = TextWidget {
            paragraph: paragraph.clone(),
            node_id: None,
            value: self.string.clone(),
        };
        let key = cx.widgets.insert(Box::new(widget));
        cx.updated.push(key);

        let measure = move |_known_dimensions, available_space: Size<AvailableSpace>| {
            let mut paragraph = paragraph.lock().unwrap();
            let max_width = match available_space.width {
                AvailableSpace::Definite(px) => px,
                AvailableSpace::MaxContent => f32::MAX,
                AvailableSpace::MinContent => f32::MIN,
            };
            paragraph.layout(max_width);

            Size {
                width: paragraph.longest_line(),
                height: paragraph.height(),
            }
        };

        let key = cx
            .taffy
            .new_leaf_with_measure(
                self.style.clone(),
                taffy::node::MeasureFunc::Boxed(Box::new(measure)),
            )
            .unwrap();
        cx.children.push(key);
    }
}

pub struct TextWidget {
    paragraph: Arc<Mutex<Paragraph>>,
    node_id: Option<NodeId>,
    value: String,
}

impl Widget for TextWidget {
    fn semantics(&mut self, cx: &mut SemanticsContext) {
        if let Some(_node_id) = self.node_id {
        } else {
            let mut classes = NodeClassSet::lock_global();
            let mut node_builder = NodeBuilder::new(Role::StaticText);
            node_builder.set_value(self.value.clone());

            let node = node_builder.build(&mut classes);

            let node_id = cx.node_id();
            self.node_id = Some(node_id);

            cx.tree_update.nodes.push((node_id, node));
            cx.node_children.push(node_id);
        }
    }
}
