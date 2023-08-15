use super::View;
use crate::Id;
use std::any::Any;

pub struct Text {}

impl From<String> for Text {
    fn from(_value: String) -> Self {
        todo!()
    }
}

impl<T, A> View<T, A> for Text {
    type State = ();

    fn view(&mut self, _state: &mut T, _id_path: &[Id], _message: Box<dyn Any>) {
        todo!()
    }
}
