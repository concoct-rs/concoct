use concoct::prelude::*;

#[derive(PartialEq)]
struct Counter {
    initial_value: i32,
}

impl View for Counter {
    fn view(&mut self) -> impl IntoView {
        let mut count = use_state(|| self.initial_value);

        (
            format!("High five count: {count}"),
            button("Up High").on_click(move || count += 1),
            button("Down low").on_click(move || count -= 1),
        )
    }
}

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    concoct::web::run(Counter { initial_value: 0 })
}
