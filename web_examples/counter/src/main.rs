use concoct::{hook::use_state, web::html, Body, View};
use wasm_bindgen_futures::spawn_local;

struct App;

impl View for App {
    fn body(&self) -> impl Body {
        let (count, set_count_high) = use_state(|| 0);
        let set_count_low = set_count_high.clone();

        let n = *count;
        (
            format!("High five count: {}", count),
            html::button("Up high!").on_click(move |_| set_count_high(n + 1)),
            html::button("Down low!").on_click(move |_| set_count_low(n - 1)),
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

    spawn_local(concoct::run(App))
}
