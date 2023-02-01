use crate::{composer::Composer, Semantics, Widget};
use accesskit::{Node, NodeId, Role};
use skia_safe::{
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    FontMgr, RGB,
};
use std::{any, panic::Location};

#[track_caller]
pub fn text(string: impl Into<String>) {
    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        if let Some(widget) = cx.get_mut::<TextWidget>(&id) {
            widget.text = string.into();
            cx.children.push(id);
        } else {
            let widget = TextWidget {
                text: string.into(),
                node_id: None,
                paragraph: None,
            };
            cx.insert(id, widget, None);
        }
    })
}

pub struct TextWidget {
    text: String,
    node_id: Option<NodeId>,
    paragraph: Option<Paragraph>,
}

impl Widget for TextWidget {
    fn semantics(&mut self, semantics: &mut Semantics) {
        let node = Node {
            role: Role::StaticText,
            value: Some(self.text.clone().into_boxed_str()),
            ..Node::default()
        };

        if let Some(node_id) = self.node_id {
            semantics.update(node_id, node);
        } else {
            let id = semantics.insert(node);
            self.node_id = Some(id);
        }

        let paragraph_style = ParagraphStyle::new();
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);

        let mut text_style = TextStyle::new();
        text_style.set_font_families(&["serif"]);
        text_style.set_color(RGB::from((0, 0, 0)));
        text_style.set_font_size(100.);
        paragraph_builder.push_style(&text_style);

        paragraph_builder.add_text(&self.text);
        paragraph_builder.pop();

        let mut paragraph = paragraph_builder.build();
        paragraph.layout(f32::MAX);
        self.paragraph = Some(paragraph);
    }

    fn paint(&mut self, _semantics: &Semantics, canvas: &mut skia_safe::Canvas) {
        self.paragraph.as_ref().unwrap().paint(canvas, (100., 100.))
    }

    fn remove(&mut self, semantics: &mut Semantics) {
        if let Some(node_id) = self.node_id {
            semantics.remove(node_id);
        }
    }

    fn any(&self) -> &dyn any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
