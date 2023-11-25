use concoct::{use_future, use_state, Composable, Composition};
use std::time::Duration;
use tokio::time;

#[derive(PartialEq)]
struct Counter {
    initial_value: i32,
}

impl Composable for Counter {
    fn compose(&mut self) {
        let mut count = use_state(|| self.initial_value);

        use_future(|| async move {
            loop {
                time::sleep(Duration::from_millis(500)).await;
                count += 1;
            }
        });

        dbg!(count);
    }
}

fn app() -> impl Composable {
    Counter { initial_value: 0 }
}

#[tokio::main]
async fn main() {
    let mut composition = Composition::new(app);
    composition.build();
    loop {
        composition.rebuild().await;
    }
}
