use concoct::{Scope, View};
use concoct_web::html;

#[derive(Default)]
struct App {
    count: i32,
}

impl View<i32> for App {
    fn body(&mut self, _cx: &Scope<i32>) -> impl View<i32> {
        (
            format!("High five count: {}", self.count),
            html::button("Up high!"),
            html::button("Down low!"),
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
}
