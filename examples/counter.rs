use concoct::{use_future, use_state, Composable, IntoComposable};

#[derive(PartialEq)]
struct Counter {
    initial_value: i32,
}

impl Composable for Counter {
    fn compose(&mut self) -> impl IntoComposable {
        let mut count = use_state(|| self.initial_value);

        use_future(|| async move {
            loop {
                //time::sleep(Duration::from_millis(500)).await;
                count += 1;
            }
        });

        dbg!(count);
    }
}

fn app() -> impl IntoComposable {
    (Counter { initial_value: 0 }, Counter { initial_value: 100 })
}

fn main() {}
