use concoct::{composable, compose, remember};

#[composable]
fn a(count: i32) -> i32 {
    count
}

#[composable]
fn app(count: i32) {
    compose!(remember(|| 0));

    dbg!(count);
}

fn main() {}
