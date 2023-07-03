use concoct::{composable, compose, remember, Composer, State};
use std::time::Duration;
use tokio::time::sleep;

#[composable]
fn counter() {
    let count = compose!(remember(|| {
        let count = State::new(0);

        let timer_count = count.clone();
        concoct::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                timer_count.update(|count| *count += 1);
            }
        });

        count
    }));

    dbg!(*count.get());
}

#[composable]
fn app() {
    compose!(remember(|| { dbg!("Ran once!") }));

    compose!(counter());
    compose!(counter());
}

#[tokio::main]
async fn main() {
    let mut composer = Composer::<(), ()>::new();
    composer.compose(app());

    loop {
        composer.recompose().await;
    }
}
