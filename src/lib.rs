pub use concoct_macros::composable;

pub trait Composable {
    type State;
    type Output;

    fn compose(self, changed: u32, state: &mut Option<Self::State>) -> Self::Output;
}

pub struct Composer<T> {
    state: Option<T>,
}

impl<T> Composer<T> {
    pub fn new() -> Self {
        Self { state: None }
    }

    pub fn compose<R>(&mut self, composable: impl Composable<State = T, Output = R>) -> R {
        composable.compose(0, &mut self.state)
    }
}
