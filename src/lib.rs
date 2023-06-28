pub use concoct_macros::composable;

pub trait Composable {
    type Input;
    type Output;

    fn compose(&mut self, changed: u32, input: Self::Input) -> Self::Output;
}