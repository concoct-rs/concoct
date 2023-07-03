use concoct::{composable, compose, remember, Composer, State};
use std::time::Duration;
use tokio::time::sleep;

#[composable]
fn counter(interval: Duration) {
    let count = compose!(remember(move || {
        let count = State::new(0);

        let timer_count = count.clone();
        concoct::spawn(async move {
            loop {
                sleep(interval).await;
                timer_count.update(|count| *count += 1);
            }
        });

        count
    }));

    dbg!(*count.get());
}

#[composable]
fn app() {
    dbg!("Ran once!");

    compose!(counter(Duration::from_secs(1)));
    compose!(counter(Duration::from_secs(2)));
}

#[tokio::main]
async fn main() {
    let mut composer = Composer::default();
    composer.compose(app());

    loop {
        composer.recompose().await;
    }
}
