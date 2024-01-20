use crate::{view::Empty, View};
use std::rc::Rc;

pub trait ViewBuilder: 'static {
    fn build(&self) -> impl View;
}

impl ViewBuilder for () {
    fn build(&self) -> impl View {
        Empty
    }
}

impl<V: ViewBuilder> ViewBuilder for Rc<V> {
    fn build(&self) -> impl View {
        (&**self).build()
    }
}

macro_rules! impl_string_view {
    ($t:ty) => {
        impl ViewBuilder for $t {
            fn build(&self) -> impl View {
                let cx = crate::hook::use_context::<crate::TextViewContext>().unwrap();
                let mut view = cx.view.borrow_mut();
                view(self.clone().into())
            }
        }
    };
}

impl_string_view!(&'static str);
impl_string_view!(String);
