#![feature(type_alias_impl_trait)]

use concoct::{composable, compose, Composer};

#[composable]
fn counter(count: i32) {
    dbg!(count);
}

#[composable]
fn app(x: i32, y: i32) {
    compose!(counter(x));

    compose!(counter(y));
}

fn main() {
    let mut composer = Composer::default();
    composer.compose(app(0, 0)); // 0, 0
    composer.compose(app(0, 0)); // Displays nothing!

    composer.compose(app(1, 0)); // 1
    composer.compose(app(1, 0)); // Displays nothing!

    composer.compose(app(0, 1)); // 0, 1
    composer.compose(app(0, 1)); // Displays nothing!
}
