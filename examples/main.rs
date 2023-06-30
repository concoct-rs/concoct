use concoct::{composable, compose, Composer};

#[composable]
fn app() {}

fn main() {
    let mut composer = Composer::new();
    composer.compose(app());
}
