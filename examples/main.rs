use concoct::{composable, compose, remember, Composer};

#[composable]
fn app() {
    let count = compose!(remember(|| 0));

    dbg!(count);
}

fn main() {
    let mut composer = Composer::new();
    composer.compose(app());
}
