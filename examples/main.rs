use concoct::{composable, compose, remember};

#[composable]
fn a(count: i32) -> i32 {
    count
}

#[composable]
fn app(x: i32) {
    let count = compose!(remember(|| 0));
    dbg!(count);
}

fn main() {}
