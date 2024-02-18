use super::Task;
use std::{any::TypeId, marker::PhantomData, rc::Rc};

pub fn context<T: 'static>() -> Context<T> {
    Context {
        _marker: PhantomData,
    }
}

pub struct Context<T> {
    _marker: PhantomData<T>,
}

impl<M, T: 'static> Task<M> for Context<T> {
    type Output = Rc<T>;

    type State = ();

    fn build(&mut self, cx: &super::Scope<M, ()>, _model: &mut M) -> (Self::Output, Self::State) {
        let rc = cx
            .contexts
            .borrow()
            .get(&TypeId::of::<T>())
            .unwrap()
            .clone();
        let output = Rc::downcast(rc).unwrap();
        (output, ())
    }

    fn rebuild(
        &mut self,
        cx: &super::Scope<M, ()>,
        _model: &mut M,
        _state: &mut Self::State,
    ) -> Self::Output {
        let rc = cx
            .contexts
            .borrow()
            .get(&TypeId::of::<T>())
            .unwrap()
            .clone();
        let output = Rc::downcast(rc).unwrap();
        output
    }
}
