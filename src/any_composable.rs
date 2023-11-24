use crate::{BuildContext, Composable};
use std::any::Any;

pub trait AnyComposable {
    fn any_build(&mut self, cx: &mut BuildContext) -> Box<dyn Any>;

    fn any_rebuild(&mut self, state: &mut dyn Any);
}

impl<C: Composable> AnyComposable for C {
    fn any_build(&mut self, cx: &mut BuildContext) -> Box<dyn Any> {
        Box::new(self.build(cx))
    }

    fn any_rebuild(&mut self, state: &mut dyn Any) {
        self.rebuild(state.downcast_mut().unwrap())
    }
}
