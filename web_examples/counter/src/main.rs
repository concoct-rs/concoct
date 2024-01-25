use concoct::{Scope, View};
use concoct_web::html;

struct App;

impl View<i32> for App {
    fn body(&mut self, _cx: &Scope<i32>) -> impl View<i32> {
        html::button("Up high!")
    }
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::TRACE)
            .build(),
    );
}
