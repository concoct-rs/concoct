use concoct::{composable, compose, Composer};

#[composable]
fn f(count: i32) {
    dbg!(count);
}

#[composable]
fn app(count: i32) {
    compose!(f(count));
}

fn main() {
    let mut composer = Composer::default();
    composer.compose(app(0));
    composer.compose(app(0));
    composer.compose(app(1));
}
