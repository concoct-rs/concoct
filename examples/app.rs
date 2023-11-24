use concoct::{use_future, use_state, Composable, Composition, Debugger};
use std::time::Duration;
use tokio::time;

fn app() -> impl Composable {
    let mut count = use_state(|| 0);

    use_future(|| async move {
        loop {
            time::sleep(Duration::from_millis(500)).await;
            count += 1;
        }
    });

    Debugger::new(count)
}

#[tokio::main]
async fn main() {
    let mut composition = Composition::new(app);
    composition.build();
    composition.rebuild().await;
    composition.rebuild().await;
}
