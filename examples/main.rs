use concoct::{composable, compose, remember, Composer};

#[composable]
fn app() {
    let count = compose!(remember(|| 0));
    dbg!(count);
}

fn main() {
    let _composer = Composer::new();
    // composer.compose(app());
}
