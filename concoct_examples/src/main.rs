#[cfg(feature = "counter")]
mod counter;

#[cfg(feature = "drop")]
mod drop;

#[cfg(feature = "text_input")]
mod text_input;

fn main() {
    #[cfg(feature = "counter")]
    let app = counter::app;

    #[cfg(feature = "drop")]
    let app = drop::app;

    #[cfg(feature = "text_input")]
    let app = text_input::app;

    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
