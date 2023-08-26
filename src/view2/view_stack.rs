use super::{BuildContext, Id};
use std::any::Any;

pub trait ViewStack<T, A = ()> {
    type Element;

    fn len(&self) -> usize;

    fn build(&mut self, cx: &mut BuildContext) -> Id;

    fn rebuild(&mut self, cx: &mut BuildContext, old: &mut Self);

    fn message(&mut self, state: &mut T, id_path: &[Id], message: &dyn Any) -> Option<A>;
}
