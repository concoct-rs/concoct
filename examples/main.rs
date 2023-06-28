use concoct::{composable, compose};

#[composable]
fn a(count: i32) -> i32 {
    count
}

#[composable]
fn b(count: i32) {
    dbg!(count);
}

fn main() {}
