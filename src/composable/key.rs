use super::Container;
use crate::{Composer, View};

/// Specify a key for a given composable.
#[track_caller]
pub fn key(key: u64, composable: impl FnMut() + 'static) {
    Composer::with(|composer| {
        composer.borrow_mut().next_key = Some(key);
    });

    Container::build_row(composable).flex_grow(1.).view()
}
