use rustc_hash::FxHasher;

use crate::{body::Empty, Body};
use std::{rc::Rc, hash::{Hash, Hasher}};

pub trait View: 'static {
    fn body(&self) -> impl Body;
}

impl View for () {
    fn body(&self) -> impl Body {
        Empty
    }
}

impl<V: View> View for Rc<V> {
    fn body(&self) -> impl Body {
        (&**self).body()
    }
}

macro_rules! impl_string_view {
    ($t:ty) => {
        impl View for $t {
            fn body(&self) -> impl Body {
                let cx = crate::hook::use_context::<crate::TextViewContext>().unwrap();
                let mut view = cx.view.borrow_mut();
                view(self.clone().into())
            }
        }
    };
}

impl_string_view!(&'static str);
impl_string_view!(String);

