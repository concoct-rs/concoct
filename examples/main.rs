use concoct::{composable, Composer};

#[composable]
fn app(count: i32) {
    dbg!(count);
}

fn main() {
    let mut composer = Composer::new();
    composer.compose(app(0));
    composer.compose(app(0));
    composer.compose(app(1));
}
