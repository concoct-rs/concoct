use concoct::{Scope, View};
use concoct_web::html;
use wasm_bindgen_futures::spawn_local;

#[derive(Default)]
struct Counter {
    count: i32,
}

impl View<Counter> for Counter {
    fn body(&mut self, _cx: &Scope<Counter>) -> impl View<Counter> {
        (
            format!("High five count: {}", self.count),
            html::button("Up high!").on_click(|state: &mut Self, _event| state.count += 1),
            html::button("Down low!").on_click(|state: &mut Self, _event| state.count -= 1),
        )
    }
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::TRACE)
            .build(),
    );

    spawn_local(concoct_web::run(Counter::default()))
}
