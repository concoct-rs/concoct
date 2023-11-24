use concoct::{use_future, use_state, Composable, Composition, Debugger};
use std::time::Duration;
use tokio::time;

fn counter() -> impl Composable {
    let mut count = use_state(|| 0);

    use_future(|| async move {
        loop {
            count += 1;
            time::sleep(Duration::from_millis(500)).await;
        }
    });

    Debugger::new(count)
}

#[tokio::main]
async fn main() {
    let mut composition = Composition::new(counter);
    composition.build();
    composition.rebuild().await;
}
