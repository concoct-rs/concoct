use concoct::{html::Div, Signal, View};

fn app() -> impl View {
    let count = Signal::new(0);

    log::info!("app");

    let count_ref = count.read();
    Div::new().view(count_ref.to_string())
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
