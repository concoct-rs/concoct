use concoct::{use_future, use_state, IntoView, View};

#[derive(PartialEq)]
struct Counter {
    initial_value: i32,
}

impl View for Counter {
    fn view(&mut self) -> impl IntoView {
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

fn app() -> impl IntoView {
    (Counter { initial_value: 0 }, Counter { initial_value: 100 })
}

fn main() {}
