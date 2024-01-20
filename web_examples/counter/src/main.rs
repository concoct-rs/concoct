use concoct::{hook::use_state, Body, View};
use concoct_web::html;
use wasm_bindgen_futures::spawn_local;

struct App;

impl View for App {
    fn body(&self) -> impl Body {
        let (count, set_high) = use_state(|| 0);
        let set_low = set_high.clone();

        (
            format!("High five count: {}", count),
            html::button("Up high!").on_click(move |_| set_high(count + 1)),
            html::button("Down low!").on_click(move |_| set_low(count - 1)),
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

    spawn_local(concoct_web::run(App))
}
