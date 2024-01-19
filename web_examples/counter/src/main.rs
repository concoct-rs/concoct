use concoct::{
    hook::{use_context, use_provider, use_state},
    web::div,
    Body, View,
};
use wasm_bindgen_futures::spawn_local;

struct Child;

impl View for Child {
    fn body(&self) -> impl Body {
        let n = use_context::<u8>();
        tracing::info!("{:?}", n);
    }
}

struct App;

impl View for App {
    fn body(&self) -> impl Body {
        let (count, set_count) = use_state(|| 0);

        let n = *count;

        use_provider(|| 42u8);

        (
            div(String::from("Up high!")).on_click(move |_| set_count(n + 1)),
            Child,
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
