use crate::{into_view::IntoView, BUILD_CONTEXT};

/// Viewable element that handles diffing.
pub trait View: PartialEq + 'static {
    fn view(&mut self) -> impl IntoView;
}

impl View for () {
    fn view(&mut self) -> impl IntoView {
        BUILD_CONTEXT
            .try_with(|cx| cx.borrow_mut().as_ref().unwrap().borrow_mut().is_done = true)
            .unwrap();
    }
}

impl View for &'static str {
    fn view(&mut self) -> impl IntoView {
        let platform = BUILD_CONTEXT
            .try_with(|cx| {
                let g = cx.borrow();
                let cx = g.as_ref().unwrap().borrow_mut();
                cx.platform.clone()
            })
            .unwrap();
        platform.from_str(self).any_view();
    }
}

impl View for String {
    fn view(&mut self) -> impl IntoView {
        log::info!("{}", self);
        let platform = BUILD_CONTEXT
            .try_with(|cx| {
                let g = cx.borrow();
                let cx = g.as_ref().unwrap().borrow_mut();
                cx.platform.clone()
            })
            .unwrap();
        platform.from_str(self).any_view();
    }
}
