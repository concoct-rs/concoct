#[cfg(feature = "counter")]
mod counter;

#[cfg(feature = "text_input")]
mod text_input;

fn main() {
    #[cfg(feature = "counter")]
    let app = counter::app;

    #[cfg(feature = "text_input")]
    let app = text_input::app;

    #[cfg(not(feature = "counter"))]
    let app: () = panic!("Please select an example with `trunk serve --features example_name`.");

    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
