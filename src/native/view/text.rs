use super::canvas;
use crate::{native::Native, View};
use skia_safe::{Color4f, Font, FontStyle, Paint, TextBlob, Typeface};
use std::borrow::Cow;

pub fn text<E>(content: impl Into<Cow<'static, str>>) -> impl View<Native<E>> {
    let content = content.into();
    canvas(move |layout, canvas| {
        let typeface = Typeface::new("Arial", FontStyle::default()).unwrap();
        let font = Font::new(typeface, Some(100.));
        let blob = TextBlob::new(&content, &font).unwrap();
        canvas.draw_text_blob(
            &blob,
            (layout.location.x + 100., layout.location.y + 100.),
            &Paint::new(Color4f::new(1., 0., 0., 1.), None),
        );
    })
}
