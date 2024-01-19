use concoct::{hook::use_state, Body, Tree, View};
use wasm_bindgen_futures::spawn_local;

struct A {
    n: u8,
}

impl View for A {
    fn body(&self) -> impl Body {
        let (count, _set_count) = use_state(|| 0);
        tracing::info!("{:?}", (self.n, &count));

        // set_count(*count + 1)
    }
}

struct App;

impl View for App {
    fn body(&self) -> impl Body {
        (A { n: 0 }, A { n: 1 })
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
