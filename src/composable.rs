use crate::{into_view::IntoView, BUILD_CONTEXT};
use std::{cell::RefCell, rc::Rc};

/// Composable object that handles diffing.
pub trait View: PartialEq + 'static {
    fn view(&mut self) -> impl IntoView;
}

impl View for () {
    fn view(&mut self) -> impl IntoView {}
}

pub struct Child<C> {
    cell: Rc<RefCell<Option<C>>>,
}

impl<C> Child<C> {
    pub fn new(composable: C) -> Self {
        Self {
            cell: Rc::new(RefCell::new(Some(composable))),
        }
    }
}

impl<C> Clone for Child<C> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<C: IntoView> IntoView for Child<C> {
    fn into_view(self) -> impl View {
        self.cell.take().unwrap().into_view()
    }
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
