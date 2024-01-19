use crate::{body::Empty, Body};
use std::rc::Rc;

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
