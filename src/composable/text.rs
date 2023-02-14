use super::widget;
use crate::{
    dimension::DevicePixels, semantics::LayoutNode, Modifier, Modify, Semantics, View, Widget,
};
use accesskit::{Node, NodeId, Role};
use skia_safe::{
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    Canvas, Color4f, FontMgr, Typeface, RGB,
};
use std::{
    any,
    sync::{Arc, Mutex},
};
use taffy::{
    node::MeasureFunc,
    prelude::{AvailableSpace, Size},
    style::Style,
};

pub struct Text<M> {
    pub modifier: M,
    pub string: String,
    pub color: Color4f,
    pub typeface: Typeface,
    pub style: Style,
    pub font_size: f32,
}

impl Text<Modifier> {
    pub fn build(string: impl Into<String>) -> Self {
        Self {
            modifier: Modifier,
            string: string.into(),
            color: Color4f::new(0., 0., 0., 1.),
            typeface: Typeface::new("serif", Default::default()).unwrap(),
            style: Style::default(),
            font_size: 14.dp(),
        }
    }

    #[track_caller]
    pub fn new(string: impl Into<String>) {
        Self::build(string).view()
    }
}

impl<M> Text<M>
where
    M: Modify + 'static,
{
    pub fn color(mut self, color: impl Into<Color4f>) -> Self {
        self.color = color.into();
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn typeface(mut self, typeface: Typeface) -> Self {
        self.typeface = typeface;
        self
    }

    pub fn modifier<M2>(self, modifier: M2) -> Text<M2> {
        Text {
            modifier,
            string: self.string,
            color: self.color,
            typeface: self.typeface,
            style: self.style,
            font_size: self.font_size,
        }
    }
}

impl<M> View for Text<M>
where
    M: Modify + 'static,
{
    #[track_caller]
    fn view(self) {
        widget(
            self,
            |text| TextWidget {
                text: text,
                node_id: None,
                layout_id: None,
                paragraph: None,
            },
            |text, node| {
                let widget: &mut TextWidget<M> = node.as_mut();
                widget.text = text;
            },
        );
    }
}

pub struct TextWidget<M> {
    text: Text<M>,
    node_id: Option<NodeId>,
    layout_id: Option<LayoutNode>,
    paragraph: Option<Arc<Mutex<Paragraph>>>,
}

impl<M> Widget for TextWidget<M>
where
    M: Modify + 'static,
{
    fn layout(&mut self, semantics: &mut Semantics) {
        let font_mgr = FontMgr::new();
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(font_mgr, None);

        let paragraph_style = ParagraphStyle::new();
        let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);

        let mut text_style = TextStyle::new();
        text_style.set_color(RGB::from((0, 0, 0)));
        text_style.set_font_size(self.text.font_size);
        text_style.set_typeface(self.text.typeface.clone());
        paragraph_builder.push_style(&text_style);

        paragraph_builder.add_text(&self.text.string);
        paragraph_builder.pop();

        let paragraph = Arc::new(Mutex::new(paragraph_builder.build()));
        self.paragraph = Some(paragraph.clone());
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

        if let Some(layout_id) = self.layout_id {
            semantics
                .taffy
                .set_style(layout_id, self.text.style)
                .unwrap();
            semantics
                .taffy
                .set_measure(layout_id, Some(MeasureFunc::Boxed(Box::new(measure))))
                .unwrap();

            semantics
                .layout_children
                .last_mut()
                .unwrap()
                .push(layout_id);
        } else {
            let layout_id = semantics.insert_layout_with_measure(self.text.style, measure);
            self.layout_id = Some(layout_id);
        }
    }

    fn semantics(&mut self, semantics: &mut Semantics) {
        let node = Node {
            role: Role::StaticText,
            value: Some(self.text.string.clone().into_boxed_str()),
            ..Node::default()
        };

        let node_id = if let Some(node_id) = self.node_id {
            semantics.update(node_id, node);
            node_id
        } else {
            let node_id = semantics.insert(node);
            self.node_id = Some(node_id);
            node_id
        };

        self.text.modifier.semantics(node_id, semantics);
    }

    fn paint(&mut self, semantics: &Semantics, canvas: &mut Canvas) {
        let layout = semantics.layout(self.layout_id.unwrap());

        self.text.modifier.paint(&layout, canvas);
        self.paragraph
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .paint(canvas, (layout.location.x, layout.location.y))
    }

    fn remove(&mut self, semantics: &mut Semantics) {
        if let Some(node_id) = self.node_id {
            semantics.remove(node_id);

            self.text.modifier.remove(node_id, semantics)
        }
    }

    fn any(&self) -> &dyn any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
