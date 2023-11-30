use concoct::{prelude::*, Platform, Tree};
use std::time::Duration;

pub struct Renderer;

impl Platform for Renderer {
    fn from_str(&self, s: &str) -> Box<dyn concoct::AnyView> {
        let state = use_ref(|| {
            dbg!(s);
            s.to_string()
        });

        if &*state.get() != s {
            dbg!(s);
            *state.get_mut() = s.to_string();
        }

        Box::new(())
    }
}

#[derive(PartialEq)]
struct Counter {
    initial_value: i32,
}

impl View for Counter {
    fn view(&mut self) -> impl IntoView {
        let mut count = use_state(|| self.initial_value);

        use_future(|| async move {
            count += 1;

            tokio::time::sleep(Duration::from_millis(500)).await;
        });

        format!("High five count: {count}")
    }
}

#[tokio::main]
async fn main() {
    let mut tree = Tree::new(Renderer, Counter { initial_value: 0 });
    tree.build();

    loop {
        tree.rebuild().await;
    }
}
