use concoct::{use_future, use_state, Composable, Composition, Debugger};
use std::time::Duration;
use tokio::time;

fn counter(initial: i32) -> impl Composable {
    let mut count = use_state(|| initial);

    use_future(|| async move {
        loop {
            count += 1;
            time::sleep(Duration::from_millis(500)).await;
        }
    });

    Debugger::new(count)
}

fn app() -> impl Composable {
    (|| counter(0), || counter(2))
}

#[tokio::main]
async fn main() {
    let mut composition = Composition::new(app);
    composition.build();
    composition.rebuild().await;
}
