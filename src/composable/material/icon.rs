pub use material_icons::Icon;

use crate::{composable::Text, Modify};
use material_icons::icon_to_char;
use skia_safe::{Data, Typeface};

thread_local! {
    pub static TYPEFACE: Typeface = {
        // Safety: BYTES has a static lifetime
        let data= unsafe { Data::new_bytes(material_icons::FONT) };
        Typeface::from_data(data, 0).unwrap()
    };
}

/// Material You icon composable
#[track_caller]
pub fn icon(modifier: impl Modify + 'static, icon: Icon, _content_description: impl Into<String>) {
    let typeface = TYPEFACE.try_with(|typeface| typeface.clone()).unwrap();

    Text::build(icon_to_char(icon))
        .typeface(typeface)
        .modifier(modifier)
        .view()
}
