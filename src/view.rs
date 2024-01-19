use crate::{body::Empty, Body};

pub trait View: 'static {
    fn body(&self) -> impl Body;
}

impl View for () {
    fn body(&self) -> impl Body {
        Empty
    }
}
