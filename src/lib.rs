pub use concoct_macros::composable;

pub trait Composable {
    type State: Default;
    type Output;

    fn compose(self, changed: u32, state: &mut Self::State) -> Self::Output;
}

#[derive(Default)]
pub struct Composer<T> {
    state: T,
}

impl<T> Composer<T> {
    pub fn compose<R>(&mut self, composable: impl Composable<State = T, Output = R>) -> R {
        composable.compose(0, &mut self.state)
    }
}

#[macro_export]
macro_rules! compose {
    ($composable:expr) => {
        $composable
    };
}
