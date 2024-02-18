use super::Task;
use std::{any::TypeId, rc::Rc};

pub fn provider<T: 'static>(value: T) -> Provider<T> {
    Provider {
        value: Rc::new(value),
    }
}

pub struct Provider<T> {
    value: Rc<T>,
}

impl<M, T: 'static> Task<M> for Provider<T> {
    type Output = Rc<T>;

    type State = ();

    fn build(&mut self, cx: &super::Scope<M, ()>, _model: &mut M) -> (Self::Output, Self::State) {
        cx.contexts
            .borrow_mut()
            .insert(TypeId::of::<T>(), self.value.clone());
        (self.value.clone(), ())
    }

    fn rebuild(
        &mut self,
        cx: &super::Scope<M, ()>,
        _model: &mut M,
        _state: &mut Self::State,
    ) -> Self::Output {
        cx.contexts
            .borrow_mut()
            .insert(TypeId::of::<T>(), self.value.clone());
        self.value.clone()
    }
}
