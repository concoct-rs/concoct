#![feature(register_tool)]
#![register_tool(concoct_rt)]

pub use concoct_macros::composable;

pub trait Composable<T> {
    type Output;

    fn compose(&mut self, input: T, changed: u32);
}