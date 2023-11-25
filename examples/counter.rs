use concoct::{composable::group, use_future, use_state, Composable, Composition};
use std::time::Duration;
use tokio::time;

#[derive(PartialEq)]
struct Counter {
    initial_value: i32,
}

impl Composable for Counter {
    fn compose(&mut self) -> impl Composable {
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
    group((Counter { initial_value: 0 }, Counter { initial_value: 100 }))
}

#[tokio::main]
async fn main() {
    let mut composition = Composition::new(app);
    composition.build();
    loop {
        composition.rebuild().await;
    }
}
