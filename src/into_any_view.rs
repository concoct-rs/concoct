use crate::{use_ref, ViewContext};
use crate::{AnyView, View};

pub trait IntoAnyView: 'static {
    fn into_any_view(self) -> Box<dyn AnyView>;
}
