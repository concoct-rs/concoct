use super::{Context, Task};
use std::marker::PhantomData;

pub struct Then<T1, F, T2, M> {
    task: T1,
    f: F,
    _marker: PhantomData<(T2, M)>,
}

impl<T1, F, T2, M> Then<T1, F, T2, M> {
    pub(super) fn new(task: T1, f: F) -> Self {
        Self {
            task,
            f,
            _marker: PhantomData,
        }
    }
}

impl<T1, F, T2, M> Task<M> for Then<T1, F, T2, M>
where
    T1: Task<M> + 'static,
    F: FnMut(&mut M, T1::Output) -> T2 + 'static,
    T2: Task<M> + 'static,
    M: 'static,
{
    type Output = T2::Output;

    type State = (T1::State, T2::State);

    fn build(&mut self, cx: &Context<M, ()>, model: &mut M) -> (Self::Output, Self::State) {
        let (output, state) = self.task.build(cx, model);
        let mut next = (self.f)(model, output);
        let (next_output, next_state) = next.build(cx, model);
        (next_output, (state, next_state))
    }

    fn rebuild(
        &mut self,
        cx: &Context<M, ()>,
        model: &mut M,
        state: &mut Self::State,
    ) -> Self::Output {
        let output = self.task.rebuild(cx, model, &mut state.0);
        let mut next = (self.f)(model, output);
        next.rebuild(cx, model, &mut state.1)
    }
}
