use crate::Composable;
use std::any::Any;

pub trait AnyComposable {
    fn any_build(&mut self) -> Box<dyn Any>;
}

impl<C: Composable> AnyComposable for C {
    fn any_build(&mut self) -> Box<dyn Any> {
        Box::new(self.compose())
    }
}
