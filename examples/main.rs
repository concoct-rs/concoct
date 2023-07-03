use concoct::{composable, compose, remember, State, Composer};

#[composable]
fn app() {
    let count = compose!(remember(|| State::new(0)));
    
    count.update(|count| *count += 1);

    dbg!(*count.get());
}

#[tokio::main]
async fn main() {
    let mut composer = Composer::<(), ()>::new();
    composer.compose(app());
    
    composer.recompose().await;
}
