use crate::{
    composable::text::{text, TextModifier},
    Modifier, Modify,
};
use skia_safe::{Data, Typeface};

use material_icons::icon_to_char;
pub use material_icons::Icon;

thread_local! {
    pub static TYPEFACE: Typeface = {
        const BYTES: &[u8] =  include_bytes!("../../../assets/MaterialIcons-Regular.ttf");

        // Safety: BYTES has a static lifetime
        let data= unsafe { Data::new_bytes(BYTES) };
        Typeface::from_data(data, 0).unwrap()
    };
}

#[track_caller]
pub fn icon(
    modifier: Modifier<TextModifier, impl Modify<TextModifier> + 'static>,
    icon: Icon,
    content_description: impl Into<String>,
) {
    let typeface = TYPEFACE.try_with(|typeface| typeface.clone()).unwrap();

    text(
        Modifier::default()
            .typeface(typeface)
            .chain(modifier.modify),
        icon_to_char(icon),
    );
}
