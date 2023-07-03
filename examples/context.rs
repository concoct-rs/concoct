use concoct::{composable, compose, context, provide, Composer};

#[composable]
fn app() {
    compose!(provide(true));

    let cx: bool = compose!(context());
    dbg!(cx);
}

fn main() {
    let mut composer = Composer::new(Box::new(()));
    composer.compose(app());
}
