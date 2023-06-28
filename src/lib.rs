pub use concoct_macros::composable;

pub trait Composable {
    type State;
    type Output;

    fn compose(self, changed: u32, state: &mut Option<Self::State>) -> Self::Output;
}