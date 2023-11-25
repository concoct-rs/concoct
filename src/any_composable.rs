use crate::{composable::IntoComposable, Composable};
use std::any::Any;

pub trait AnyComposable {
    fn as_any(&self) -> &dyn Any;

    fn any_build(&mut self) -> Box<dyn Any>;

    fn any_eq(&self, other: &dyn Any) -> bool;
}

impl<C: Composable> AnyComposable for C {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn any_build(&mut self) -> Box<dyn Any> {
        Box::new(self.compose().into_composer())
    }

    fn any_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<C>() {
            self == other
        } else {
            false
        }
    }
}
