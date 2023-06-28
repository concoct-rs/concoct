use std::any::TypeId;

pub use concoct_macros::composable;

pub trait Compose {
    fn start_restart_group(&mut self, type_id: TypeId);

    fn end_restart_group(&mut self, f: impl FnOnce() -> Box<dyn FnMut(&mut Self)>);
}

pub trait Composable {
    type Output;

    fn compose(self, compose: &mut impl Compose, changed: u32) -> Self::Output;
}

#[macro_export]
macro_rules! compose {
    ($composable:expr) => {
        $composable
    };
}
