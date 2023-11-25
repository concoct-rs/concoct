use crate::{BuildContext, Composable};
use std::any::Any;

pub trait AnyComposable {
    fn any_build(&mut self, cx: &mut BuildContext) -> Box<dyn Any>;
}

impl<C: Composable> AnyComposable for C {
    fn any_build(&mut self, cx: &mut BuildContext) -> Box<dyn Any> {
        Box::new(self.compose(cx))
    }
}
