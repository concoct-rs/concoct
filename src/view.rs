use crate::{into_view::IntoView, BUILD_CONTEXT};
use std::{cell::RefCell, rc::Rc};

/// Viewable element that handles diffing.
pub trait View: PartialEq + 'static {
    fn view(&mut self) -> impl IntoView;
}

impl View for () {
    fn view(&mut self) -> impl IntoView {}
}



impl View for &'static str {
    fn view(&mut self) -> impl IntoView {
        BUILD_CONTEXT
            .try_with(|cx| {
                let g = cx.borrow();
                let mut cx = g.as_ref().unwrap().borrow_mut();
                cx.platform.from_str(self).any_view()
            })
            .unwrap();
    }
}
