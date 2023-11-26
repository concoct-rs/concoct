use crate::{IntoView, View};
use std::any::Any;

pub trait AnyView {
    fn as_any(&self) -> &dyn Any;

    fn any_view(&mut self) -> Box<dyn Any>;

    fn any_eq(&self, other: &dyn Any) -> bool;
}

impl<C: View> AnyView for C {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn any_view(&mut self) -> Box<dyn Any> {
        Box::new(self.view().into_view())
    }

    fn any_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<C>() {
            self == other
        } else {
            false
        }
    }
}
