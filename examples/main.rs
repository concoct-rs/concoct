use concoct::{composable, compose};

#[composable]
fn counter(count: i32) {
    dbg!(count);
}

#[composable]
fn app(x: i32, y: i32) {
    compose!(counter(x));

    compose!(counter(y));
}

fn main() {}
