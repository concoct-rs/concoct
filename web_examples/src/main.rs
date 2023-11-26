use concoct::{use_state, web::div, IntoView, View};

#[derive(PartialEq)]
struct Counter {
    initial_value: i32,
}

impl View for Counter {
    fn view(&mut self) -> impl IntoView {
        let mut count = use_state(|| self.initial_value);

        (
            "High five count: {count}",
            div("Up High").on_click(|| count += 1),
            div("Down low").on_click(|| count -= 1),
        )
    }
}

fn main() {
    concoct::web::run(Counter { initial_value: 0 })
}
