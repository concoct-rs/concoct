use crate::Modify;

pub struct Modifier;

impl<T> Modify<T> for Modifier {
    fn modify(&mut self, _value: &mut T) {}
}
