use crate::{into_view::IntoView, ViewContext};

/// Viewable element that handles diffing.
pub trait View: PartialEq + 'static {
    fn view(&mut self) -> impl IntoView;
}

impl View for () {
    fn view(&mut self) -> impl IntoView {
        ViewContext::current().inner.borrow_mut().is_done = true;
    }
}

impl View for &'static str {
    fn view(&mut self) -> impl IntoView {
        let platform = ViewContext::current().inner.borrow().platform.clone();
        platform.text(self).any_view();
    }
}

impl View for String {
    fn view(&mut self) -> impl IntoView {
        let platform = ViewContext::current().inner.borrow().platform.clone();
        platform.text(self).any_view();
    }
}
