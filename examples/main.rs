use concoct::{composable, compose, remember};

#[composable]
fn app() {
    let count = compose!(remember(|| 0));
    dbg!(count);
}

fn main() {}
